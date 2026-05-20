import React from 'react'

export function NewsCard({ data }) {
  const time = data.published_at ? new Date(data.published_at).toLocaleTimeString() : 'N/A'
  const sentimentClass = data.sentiment === 'positive' ? 'sentiment-pos' : data.sentiment === 'negative' ? 'sentiment-neg' : ''
  
  return (
    <div className="pulse-card news-card animate-in">
      <div className="card-top">
        <span className="source-tag">{data.source_name}</span>
        <span className="time-tag">{time}</span>
      </div>
      <h4 className="pulse-title">
        <a href={data.url} target="_blank" rel="noreferrer">{data.original_title}</a>
      </h4>
      {data.summary && <p className="pulse-summary">{data.summary}</p>}
      <div className="card-bottom">
        {data.impact_level && (
          <span className={`impact-badge impact-${data.impact_level.toLowerCase()}`}>
            {data.impact_level.toUpperCase()} IMPACT
          </span>
        )}
        {data.sentiment && (
          <span className={`sentiment-badge ${sentimentClass}`}>
            {data.sentiment.toUpperCase()}
          </span>
        )}
      </div>
    </div>
  )
}

export function EquityCard({ data }) {
  const time = data.published_at ? new Date(data.published_at).toLocaleTimeString() : 'N/A'
  
  return (
    <div className="pulse-card equity-card animate-in">
      <div className="card-top">
        <span className="source-tag equity">{data.source_name}</span>
        <span className="time-tag">{time}</span>
      </div>
      <div className="ticker-strip">
        {data.tickers?.map(t => (
          <span key={t} className="ticker-badge">{t}</span>
        ))}
      </div>
      <h4 className="pulse-title">
        <a href={data.url} target="_blank" rel="noreferrer">{data.title}</a>
      </h4>
      <div className="card-bottom">
        <span className={`impact-badge impact-${(data.impact_level || 'low').toLowerCase()}`}>
          {(data.impact_level || 'LOW').toUpperCase()}
        </span>
        <span className="category-badge">{data.category || 'EQUITY'}</span>
      </div>
    </div>
  )
}

export function XPostCard({ data }) {
  const time = data.created_at ? new Date(data.created_at).toLocaleTimeString() : 'N/A'
  
  return (
    <div className="pulse-card x-card animate-in">
      <div className="x-header">
        <div className="x-author-info">
          {data.author_avatar && <img src={data.author_avatar} alt="" className="x-avatar" />}
          <div className="x-author-details">
            <span className="x-author-name">{data.author_name}</span>
            <span className="x-author-user">@{data.author_username}</span>
          </div>
        </div>
        <span className="time-tag">{time}</span>
      </div>
      <p className="x-text">{data.text}</p>
      {data.media_urls?.[0] && (
        <img src={data.media_urls[0]} alt="" className="x-media" />
      )}
      <div className="card-bottom">
        <a href={data.url} target="_blank" rel="noreferrer" className="x-link">View on X</a>
      </div>
    </div>
  )
}
