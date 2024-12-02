## 常用命令

1. 发现无用的依赖
```shell
cargo +nightly udeps
```

2. 构建项目镜像
```shell
docker build --tag mailnewsletter -f Dockerfile .
```

3. 清除项目构建
```shell
cargo clean
```

## 项目的登录密码

everythinghastostartsomewhere