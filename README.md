# Mist
This is the repository for Mist, the official helper for the makedeb Package Repository.

[![Latest deployment status](https://img.shields.io/drone/build/makedeb/mist?logo=drone&server=https%3A%2F%2Fdrone.hunterwittenborn.com)](https://drone.hunterwittenborn.com/makedeb/mist/latest)

## Installation
Users have a few options for installing Mist:

### From the Prebuilt-MPR
This is the recommended way to install Mist. It avoids the need to compile any software, allows for automatic upgrades via APT, and gets you set up in just a couple of minutes.

First, [set up the Prebuilt-MPR on your system](https://docs.makedeb.org/prebuilt-mpr/getting-started), then just run the following to install Mist:

```sh
sudo apt install mist
```

### From the MPR
You can also install Mist directly from the MPR if you'd prefer that.

To install from source, run the following:

```sh
git clone 'https://mpr.makedeb.org/mist'
cd mist/
makedeb -si
```

> Mist currently requires the nightly version of the Rust compiler toolchain in order to build. To build it locally, it's recommended to use [rustup](https://rustup.rs), which will automatically manage and update the nightly toolchain on your local system. If preferred, rustup can be installed from the [MPR](https://mpr.makedeb.org/packages/rustup) or the Prebuilt-MPR.

## Contributing
If there's something you want added/fixed in Mist, feel free to open a pull request. There aren't many guidelines on what you should do quite yet, so just submit your changes and we can figure out what to do from there!
