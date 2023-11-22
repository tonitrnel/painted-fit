# Fit Protocol 解码/编码库

## 概述

解析 Garmin Fit 格式的文件，该项目受到以下项目启发：

[https://github.com/garmin/fit-javascript-sdk](https://github.com/garmin/fit-javascript-sdk)

[https://github.com/stadelmanma/fitparse-rs](https://github.com/stadelmanma/fitparse-rs)

## 用法

### 解码

参考 [examples/decode](examples/decode)

### 编码

尚未实现

## 已知问题

- `Compressed Timestamp` 由于缺少可用的测试数据因此可能无法正常工作
- `Developer Fields` 开发人员字段解析已实现但是没有输出
- `Units Profile` 文件提供的 Units 字段已解析但是没有输出

## 更新 Profile

该项目 `profile` 目录下的 `messages.rs`、`types.rs`、`version.rs` 由工具自动生成，运行该工具的命令为：

```shell
cargo run --package profile-gen --bin profile-gen -- -p "<FitSDKRelease_xx.xxx.xx.zip>"
```

`FitSDKRelease` 可在 Garmin 官方开发者网站下载，网站地址是：[https://developer.garmin.com/fit/download/](https://developer.garmin.com/fit/download/)

`profile-gen` 的路径支持 FitSDKRelease Zip 压缩包、FitSDKRelease 解压缩的输出目录和 `Profile.xlsx` 文件，但是注意，如果路径为 `Profile.xlsx` 文件则需要使用`--sdk-version`来指定 sdk 版本