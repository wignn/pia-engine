"""Official setup.py for piaa-sdk, ensuring backward-compatibility across all pip versions."""

from setuptools import find_packages, setup

with open("README.md", "r", encoding="utf-8") as f:
    long_description = f.read()

setup(
    name="piaa-sdk",
    version="1.0.1",
    description="Official Python SDK for PIA Market Intelligence & Realtime Financial Platform",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Merrr",
    author_email="wign@wign.dev",
    url="https://pia.wign.dev/portal/docs",
    project_urls={
        "Repository": "https://github.com/wignn/atlsd",
        "Issues": "https://github.com/wignn/atlsd/issues",
    },
    packages=find_packages(include=["pia*", "pia_sdk*"]),
    package_data={
        "pia": ["py.typed"],
        "pia_sdk": ["py.typed"],
    },
    include_package_data=True,
    python_requires=">=3.8",
    install_requires=[
        "httpx>=0.24.0",
    ],
    extras_require={
        "realtime": ["websockets>=11.0.0"],
        "all": ["websockets>=11.0.0"],
    },
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Financial and Insurance Industry",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Typing :: Typed",
    ],
    keywords=["pia", "atlsd", "market-data", "finance", "crypto", "trading", "sdk", "websocket"],
)
