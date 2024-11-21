## 常用命令

1. 发现无用的依赖
```shell
cargo +nightly udeps
```

2. 构建项目镜像
```shell
docker build --tag mailnewsletter -f Dockerfile .
```