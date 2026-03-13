#!/bin/bash
cd ../seshat-dj2
sed -i 's/return Err(EndpointError::InvalidContentType(content_type.to_string()));/println!("Invalid CT: {}", content_type); return Err(EndpointError::InvalidContentType(content_type.to_string()));/' diagram_tool/src/backend.rs
cargo test --lib backend::tests::p4_violation -- --nocapture
git restore diagram_tool/src/backend.rs
