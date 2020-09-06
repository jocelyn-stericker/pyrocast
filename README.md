<img alt="Pyrocast logo" src="https://github.com/jnetterf/pyrocast/blob/master/icon/logo_mark.png" height="160" />

> This is a **work in progress**. I currently recommend that you use [GNOME Podcasts](https://gitlab.gnome.org/World/podcasts).

Pyrocast is a podcast app for Linux phones that use gtk.

Features:
 - Browse or search iTunes podcasts.
 - Stream podcasts.

## Building

You need to have [Rust installed](https://www.rust-lang.org/learn/get-started).

You also need to have headers for gtk, libhandy, and gstreamer installed, as well as gst-plugins-bad.
The instructions for doing so vary by OS (please feel free to add specifics here).
In Ubuntu 20.04:

```
sudo apt-get install -y libgtk-3-dev libgstreamer1.0-dev libgstreamer-plugins-bad1.0-dev libhandy-0.0-dev
```

To build & run,

```
cargo run
```

## Cross-compiling

The best method to cross compile will vary by what OS you use on your computer and what OS you use on your phone.

Here's what worked for me with [the unofficial Arch Linux ARM for Pinephone](https://github.com/dreemurrs-embedded/Pine64-Arch) on my phone:
 - Setup ssh on your phone (use another user and be sure to disable ssh for "alarm", which probably uses an insecure password).
 - Get it to build on the phone.
 - Copy everything over: `rsync -avz joshua@192.168.0.48:/ /home/joshua/pine-sysroot`

Then, build it. I recommend building it in release mode to minimize the size of the binary you need to copy over.
```
env CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
	CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="/linux-runner aarch64" \
	CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
	CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
	PKG_CONFIG_PATH= \
	PKG_CONFIG_LIBDIR=/home/joshua/pine-sysroot/usr/lib/pkgconfig:/home/joshua/pine-sysroot/usr/share/pkgconfig \
	PKG_CONFIG_SYSROOT_DIR=/home/joshua/pine-sysroot \
	LD_LIBRARY_PATH=/home/joshua/pine-sysroot/usr/lib \
	RUSTFLAGS="-C link-arg=--sysroot -C link-arg=/home/joshua/pine-sysroot" \
	cargo build --target aarch64-unknown-linux-gnu --release
```

Then, copy it over (the `-C` uses compression, which can be much faster):
```
scp -C ./target/aarch64-unknown-linux-gnu/release/pyrocast joshua@192.168.0.48:
```

On your phone, you can run it like:
```
~joshua/pyrocast
```

## Contributing

Please do. Bugfixes and new features are very welcome.

## Special thanks & prior art

 - [Gnome Podcasts](https://gitlab.gnome.org/World/podcasts/-/tree/master/podcasts-gtk)
 - [vgtk](https://github.com/bodil/vgtk)
