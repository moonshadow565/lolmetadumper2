Dumping league metaclasses  

Build instructions: 
```
rustup target add i686-pc-windows-msvc
cargo build --release
```

Usage Instructions:
```
# Download (or copy) league .exe and .dll's
fckrman dl manifest.manifest -o Game -p '.+\.(dll|exe)'

# Copy built BugSplat.dll into Game folder overriding the exsiting one
cp build/target/i686-pc-windows-msvc/release/BugSplat.dll BugSplat.dll

# Start league (use double click on windows instead of wine)
wine League\ of\ Legends.exe
```

