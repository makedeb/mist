# Mist
[![Latest deployment status](https://img.shields.io/drone/build/makedeb/mist?logo=drone&server=https%3A%2F%2Fdrone.hunterwittenborn.com)](https://drone.hunterwittenborn.com/makedeb/mist/latest)
[![MPR - mist](https://img.shields.io/badge/mpr-mist-orange)](https://mpr.makedeb.org/packages/mist)
[![MPR - mist-bin](https://img.shields.io/badge/mpr-mist--bin-orange)](https://mpr.makedeb.org/packages/mist-bin)

This is the repository for Mist, the official command-line interface for the makedeb Package Repository.

Mist makes it easier for users to interact with the MPR in a variety of ways. Some of its most notable features include:

- Functioning as a wrapper around APT, adding in MPR functionality such as:
- - The ability to install, upgrade, and remove both APT and MPR packages.
- - Automatic dependency resolution for packages from the MPR, as well as APT.
- - Searching and listing both APT and MPR packages.
- Cloning packages from the MPR.
- Listing comments for packages from the MPR.
- Commenting packages from the MPR.

## Installation
Users have a few options for installing Mist:

### From the Prebuilt-MPR (Recommended)
This is the recommended way to install Mist. It avoids the need to compile any software, allows for automatic upgrades via APT (and Mist once it's installed), and gets you set up in just a couple of minutes.

First, [set up the Prebuilt-MPR on your system](https://docs.makedeb.org/prebuilt-mpr/getting-started), then just run the following to install Mist:

```sh
sudo apt install mist
```

### From the MPR
You can also install Mist directly from the MPR if you'd prefer that.

#### From Source
To install from source, install `mist` from the MPR:

```sh
git clone 'https://mpr.makedeb.org/mist'
cd mist/
makedeb -si -H 'MPR-Package: yes'
```

> If you omit `-H 'MPR-Package: yes'`, Mist will be **unable to update itself**.

> Mist currently requires the nightly version of the Rust compiler toolchain in order to build. To build it locally, it's recommended to use [rustup](https://rustup.rs), which will automatically manage and update the nightly toolchain on your local system. If preferred, rustup can be installed from the [MPR](https://mpr.makedeb.org/packages/rustup) or the Prebuilt-MPR.

#### From a Binary
To install Mist from a prebuilt binary, install the `mist-bin` package:

```sh
git clone 'https://mpr.makedeb.org/mist-bin'
cd mist/
makedeb -si
```

## Contributing
If there's something you want added/fixed in Mist, feel free to open a pull request. There aren't many guidelines on what you should do quite yet, so just submit your changes and we can figure out what to do from there!
