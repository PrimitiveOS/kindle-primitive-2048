# Install 2048

After [setting up PrimitiveOS](https://github.com/DoanNguyenTrong/kindle-primitive-launcher/blob/main/docs/INSTALL.md#first-time-setup-over-usb), open **Store** from Launcher.
Find **2048** under **Apps**, tap **Get**, then open it from the Launcher grid.
If you already installed Tidefork, tap **Update**. Your existing progress is available under **Line puzzles**.

## Build a local package

Run from this checkout with Tools, Launcher, and Build next to it.

```sh
cargo zigbuild --offline --locked --release -p tidefork-kindle --target armv7-unknown-linux-musleabihf
sh scripts/stage.sh armhf
python3 ../doannt-kindle-build/scripts/package.py --id primitive-tidefork --dir tidefork \
  --version 0.1.1 --arch armhf --stage build/stage --license LICENSE \
  --docs-license LICENSE-DOCS --install INSTALL.md --out dist
```
