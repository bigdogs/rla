# Rust Love Android

- Delegate to apksigner/smali/baksmali

  [jdax](https://github.com/skylot/jadx/releases) has many too dependencies, `rla` doesn't integrate it to binary but use it as command directly if needed. you need to make sure it is installed properly.

  install jadx from brew will install openjdk by default :(, if you want to use (orcacle jdk)[https://www.oracle.com/java/technologies/downloads/]

  ```shell
    brew install jadx
    brew uninstall --ignore-dependencies java
    # edit JAVA_HOME at `which jadx` & `which jadx-gui`
  ```
