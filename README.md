# Mari
This is the repository for Mari, the official helper for the makedeb Package Repository.

[![Latest deployment status](https://img.shields.io/drone/build/makedeb/mari?logo=drone&server=https%3A%2F%2Fdrone.hunterwittenborn.com)](https://drone.hunterwittenborn.com/makedeb/mari/latest)

## Installation
Users have a few options for installing Mari:

### From the Prebuilt-MPR
This is the recommended way to install Mari. It avoids the need to compile any software, allows for automatic upgrades via APT, and gets you set up in just a couple of minutes.

First, [set up the Prebuilt-MPR on your system](https://docs.makedeb.org/prebuilt-mpr/getting-started), then just run the following to install Mari:

```sh
sudo apt install mari
```

### From the MPR
You can also install Mari directly from the MPR if you'd prefer that.

To install from source, run the following:

```sh
git clone 'https://mpr.makedeb.org/mari'
cd mari/
makedeb -si
```

> Mari needs the latest version of the Rust compiler toolchain in order to build. It may work with older releases, but they're not tested against and aren't guaranteed to work. If you're system's repositories don't contain the latest release, the Rust toolchain can be installed from the [MPR](https://mpr.makedeb.org/packages/rustc) or the Prebuilt-MPR.

## Contributing
If there's something you want added/fixed in Mari, feel free to open a pull request. There aren't many guidelines on what you should do quite yet, so just submit your changes and we can figure out what to do from there!
