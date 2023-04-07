Dumping league metaclasses

Build instructions:
```
rustup target add x86_64-pc-windows-msvc
cargo build --release
```

Usage Instructions:
```
# Download (or copy) league .exe and .dlls
fckrman dl manifest.manifest -o Game -p '.+\.(dll|exe)'

# Copy built TextShaping.dll into Game folder
cp target/x86_64-pc-windows-msvc/release/TextShaping.dll Game/TextShaping.dll

# Start league via double click or running from command line
League\ of\ Legends.exe
```
