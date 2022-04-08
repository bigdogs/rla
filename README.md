# Rust Love Android

## prerequisite

- [java](https://www.oracle.com/java/technologies/downloads/) should be available as command
- [jadx](https://github.com/skylot/jadx/releases) is used by some commands, but it's optional

installing jadx by homebrew will download openjdk, you need to uninstall it if you want to use oracle jdk

```shell
  brew install jadx
  brew uninstall --ignore-dependencies java
  # edit `which jadx` && `which jadx-gui` to remove JAVA_HOME check
```

## usage

```
rla --help
```
