<p align="center"><a href="https://github.com/Shubhamai/StableView"><img alt="StableView" src="assets/brand/banner.png" width="100%"/></a></p>
<p align="center"><em>Background image credits - Les Chevaliers du Ciel</em></p>

<hr>

<p align="center"> Easy, fast and efficient Head Tracking application using only webcam </p>

<p align="center">
<a href="https://github.com/shubhamai/StableView/releases/latest"><img alt="" src="https://badgen.net/badge/Download/Windows/?color=blue&icon=windows&label"/></a> 
<a href="https://github.com/Shubhamai/StableView/actions/workflows/ci.yml"><img alt="" src="https://github.com/Shubhamai/StableView/actions/workflows/ci.yml/badge.svg"/></a>
<a href="https://github.com/Shubhamai/StableView/actions/workflows/audit.yml"><img alt="" src="https://github.com/Shubhamai/StableView/actions/workflows/audit.yml/badge.svg"/></a>
<a href="https://github.com/shubhamai/StableView/blob/main/LICENSE"><img alt="" src="https://img.shields.io/badge/License-MIT-blue.svg"/></a>
<a href="https://github.com/shubhamai/StableView/releases/latest"><img alt="" src="https://img.shields.io/github/downloads/shubhamai/StableView/total.svg?style=flat"/></a>
</p>

<div align="center">
  <h3>
    <a href="https://github.com/shubhamai/StableView/releases/">
      Download
    </a>
    <span> | </span>
    <a href="https://github.com/shubhamai/StableView/wiki">
      Wiki
    </a>
    <span> | </span>
    <a href="https://github.com/Shubhamai/StableView/discussions">
      Chat
    </a>
    <span> | </span>
    <a href="https://github.com/Shubhamai/StableView/blob/main/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>

<hr>

# Status

**Last Updated - 6 June, 2024**

![progress gif](assets/updates/may-12-2023-update.gif)

- Recent Updates
  - [x] Supporting Linux and Apple Silicon
  - [x] Fixing bug causing reduced performance #78

# Usage

1. Visit the [releases page](https://github.com/shubhamai/StableView/releases/latest) and download the latest version on your platform. For Windows, a `.msi` installer will be provided, simply double-click on the installer and follow the installation steps. After Installing, you can simply run `StableView` from the start menu.

   - Make sure you have internet connectivity while installing the application as it downloads the model weights for the first time.

2. The application uses opentrack to send the tracking data to respective applications. Please install it from their [Github repo](https://github.com/opentrack/opentrack).

   After installing OpenTrack, select Input as **UDP over network** so that OpenTrack can receive data from StableView and send it to the required application.

### Linux

Run the following command in the terminal inside the folder:

```bash
LD_LIBRARY_PATH=. ./StableView
```

### MacOS ( Apple Silicon )

Run the following command in the terminal inside the folder:

```bash
DYLD_FALLBACK_LIBRARY_PATH=. ./StableView
```


# Features

- Uses your regular old webcam with AI for head tracking. Uses an extremely low CPU (<3%-60fps in Ryzen 5 3600H) and returns high performance.
- Works with [opentrack](https://github.com/opentrack/opentrack) to run on any modern simulator including Microsoft Flight Simulator, Digital Combat Simulator, Xplane & more.
- Easy to install :)

# Shoutouts

- Thanks to the authors of the paper [3DDFA_V2 : Towards Fast, Accurate and Stable 3D Dense Face Alignment](https://paperswithcode.com/paper/towards-fast-accurate-and-stable-3d-dense-1), without them, this application wouldn't have been possible, the majority of the model inference code is based on their work. Thanks, [Jianzhu Guo](https://guojianzhu.com), [Xiangyu Zhu](http://www.cbsr.ia.ac.cn/users/xiangyuzhu/), [Yang Yang](http://www.cbsr.ia.ac.cn/users/yyang/main.htm), Fan Yang, [Zhen Lei](http://www.cbsr.ia.ac.cn/users/zlei/) and [Stan Z. Li](https://scholar.google.com/citations?user=Y-nyLGIAAAAJ).
- [Rust Faces](https://github.com/rustybuilder/rust-faces) by [rustybuilder](https://github.com/rustybuilder/rust-faces) face detection in rust, used to recapture the face when it's lost.
- [Sniffer](https://github.com/GyulyVGC/sniffnet/) for GUI inspirations, code structure, readme, etc.
- [ChatGPT](https://openai.com/blog/chatgpt/) for assisting me to convert some of the Python code to Rust.
- Product Icon from [Leonardo Yip](https://unsplash.com/@yipleonardo) on [Unsplash](https://unsplash.com/photos/rn-NLirHQPY).

## Bug Report

If you see an error message or run into an issue, please [open a new issue](https://github.com/Shubhamai/StableView/issues/new/choose). This effort is valued and helps all the users.

## Feature Request

If you have any idea or a missing feature you would like to see, please [submit a feature request](https://github.com/Shubhamai/StableView/issues/new/choose) or [discuss](https://github.com/Shubhamai/StableView/discussions) it with other users.

## Contributing

Contributions are greatly appreciated! If you want to contribute to the project, please read [Contributing.md](CONTRIBUTING.md) for more details.

## Contributors

Thanks to all the people who contributed to the project.

<a href="https://github.com/shubhamai/StableView/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=shubhamai/StableView&max=30&columns=10" />
</a>

## License

StableView is open-source and free software released under the [MIT License](LICENSE).

## Citations

```bibtex
@inproceedings{guo2020towards,
    title =        {Towards Fast, Accurate and Stable 3D Dense Face Alignment},
    author =       {Guo, Jianzhu and Zhu, Xiangyu and Yang, Yang and Yang, Fan and Lei, Zhen and Li, Stan Z},
    booktitle =    {Proceedings of the European Conference on Computer Vision (ECCV)},
    year =         {2020}
}
```

```bibtex
@misc{3ddfa_cleardusk,
    author =       {Guo, Jianzhu and Zhu, Xiangyu and Lei, Zhen},
    title =        {3DDFA},
    howpublished = {\url{https://github.com/cleardusk/3DDFA}},
    year =         {2018}
}
```
