from __future__ import annotations

from collections.abc import Mapping

import torch
from torch import Tensor, nn


class MultimodalForecaster(nn.Module):
    def __init__(
        self,
        price_features: int,
        modality_dims: Mapping[str, int],
        quantiles: tuple[float, ...] = (0.1, 0.5, 0.9),
        hidden_dim: int = 64,
    ) -> None:
        super().__init__()
        self.quantiles = quantiles
        self.price_projection = nn.Linear(price_features, hidden_dim)
        encoder_layer = nn.TransformerEncoderLayer(
            d_model=hidden_dim,
            nhead=4,
            dim_feedforward=hidden_dim * 2,
            dropout=0.1,
            batch_first=True,
            norm_first=True,
        )
        self.price_encoder = nn.TransformerEncoder(encoder_layer, num_layers=2)
        positions = torch.arange(256, dtype=torch.float32).unsqueeze(1)
        dimensions = torch.arange(0, hidden_dim, 2, dtype=torch.float32)
        frequencies = torch.exp(dimensions * (-torch.log(torch.tensor(10_000.0)) / hidden_dim))
        positional = torch.zeros(256, hidden_dim)
        positional[:, 0::2] = torch.sin(positions * frequencies)
        positional[:, 1::2] = torch.cos(positions * frequencies)
        self.position = nn.Parameter(positional.unsqueeze(0))
        self.modality_encoders = nn.ModuleDict(
            {
                name: nn.Sequential(
                    nn.Linear(size, hidden_dim),
                    nn.GELU(),
                    nn.LayerNorm(hidden_dim),
                    nn.Linear(hidden_dim, hidden_dim),
                    nn.GELU(),
                )
                for name, size in modality_dims.items()
                if size > 0
            }
        )
        self.modality_gates = nn.ModuleDict({name: nn.Linear(hidden_dim * 2, 1) for name in self.modality_encoders})
        self.cross_attention = nn.MultiheadAttention(hidden_dim, num_heads=4, dropout=0.1, batch_first=True)
        self.fusion_norm = nn.LayerNorm(hidden_dim)
        self.direction_head = nn.Linear(hidden_dim, 3)
        self.quantile_head = nn.Linear(hidden_dim, len(quantiles))
        self.uncertainty_head = nn.Linear(hidden_dim, 1)

    def forward(self, batch: Mapping[str, object]) -> dict[str, Tensor]:
        price = batch["price_seq"]
        assert isinstance(price, Tensor)
        length = min(price.shape[1], self.position.shape[1])
        price_context = self.price_projection(price[:, -length:]) + self.position[:, :length]
        price_context = self.price_encoder(price_context)[:, -1:, :]

        tokens = [price_context[:, 0, :]]
        masks = [torch.ones(price.shape[0], dtype=torch.bool, device=price.device)]
        modalities = batch.get("modalities", {})
        modality_masks = batch.get("masks", {})
        assert isinstance(modalities, Mapping) and isinstance(modality_masks, Mapping)
        for name, encoder in self.modality_encoders.items():
            values = modalities.get(name)
            mask = modality_masks.get(name)
            if not isinstance(values, Tensor) or not isinstance(mask, Tensor):
                continue
            token = encoder(values)
            gate = torch.sigmoid(self.modality_gates[name](torch.cat([price_context[:, 0, :], token], dim=-1)))
            tokens.append(token * gate)
            masks.append(mask.to(dtype=torch.bool))
        key_values = torch.stack(tokens, dim=1)
        valid_tokens = torch.stack(masks, dim=1)
        attended, _ = self.cross_attention(
            query=price_context,
            key=key_values,
            value=key_values,
            key_padding_mask=~valid_tokens,
            need_weights=False,
        )
        fused = self.fusion_norm(price_context[:, 0, :] + attended[:, 0, :])
        raw_quantiles = self.quantile_head(fused)
        if raw_quantiles.shape[-1] > 1:
            increments = torch.nn.functional.softplus(raw_quantiles[:, 1:])
            return_quantiles = torch.cat([raw_quantiles[:, :1], raw_quantiles[:, :1] + increments.cumsum(dim=-1)], dim=-1)
        else:
            return_quantiles = raw_quantiles
        return {
            "direction_logits": self.direction_head(fused),
            "return_quantiles": return_quantiles,
            "uncertainty": torch.nn.functional.softplus(self.uncertainty_head(fused)).squeeze(-1),
        }
