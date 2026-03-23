# Ratatui on Nintendo Switch

Standing on the shoulders of giants, I was able to cobble together a working example of Ratatui running on the Nintendo Switch.

<p align=middle>
<img src=media/battery-charge-with-overlay.jpg width=400>
<img src=media/touch.jpg width=400>
</p>

It relies on [Mousefood](https://github.com/ratatui/mousefood) and [nx](https://github.com/aarch64-switch-rs/nx) (though I've had to fork and update the `embedded-graphics-core` version).

## Run it yourself

You have to have a modded Nintendo Switch, with the ability to run `.nro`:s (homebrew apps).

Then use [`cargo-nx`](https://github.com/aarch64-switch-rs/cargo-nx) to build and upload the code to your Switch:

1. Build (you MUST do a `release` build. `debug` is simply too unoptimized):

   ```sh
   cargo nx build --release --package <EXAMPLE NAME>
   ```

2. Start some form of `nxlink` receiver on the Nintendo Switch. My homebrew menu of choice [CyberLink](https://github.com/luketanti/CyberFoil) already has one running in the background.

3. Upload (Switch must be in same local network):
   ```sh
   cargo nx link <LOCATION OF BUILT .nro>
   ```

Instead of steps 2-3, you could also manually copy the `.nro` to the correct location in the SD card.
