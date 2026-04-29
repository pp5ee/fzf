## Goal
编写一个 Ubuntu 脚本：从源码编译 `fzf` 的 release 版本并安装到系统中，使其在命令行中的使用体验尽量接近通过 `apt install fzf` 安装后的效果。

## Context
用户希望通过脚本在 Ubuntu 上完成 `fzf` 的源码编译、安装与基础集成，而不是直接依赖系统包管理器安装。  
“像 go 版本 apt install 后一样的效果”可理解为：
- 可直接通过 `fzf` 命令使用二进制；
- 安装必要的 shell 集成脚本（如 bash/zsh 补全与快捷键）；
- 安装位置与环境变量处理应尽量符合 Ubuntu 常规使用方式。

由于未提供现有工作区代码，本计划按全新脚本实现设计。

## Acceptance Criteria
- AC-1: 提供一个可在 Ubuntu 上执行的脚本，自动完成依赖检查、获取 `fzf` 源码、编译 release 二进制并安装。
- AC-2: 安装完成后，执行 `fzf --version` 能返回成功，且命令在新 shell 会话中可直接使用。
- AC-3: 脚本将二进制安装到标准可执行路径（如 `/usr/local/bin`）或明确配置到 `PATH` 中。
- AC-4: 脚本能够安装或启用常见 shell 集成资源，使 bash/zsh 用户的体验尽量接近包管理器安装后的默认效果。
- AC-5: 脚本对缺失依赖、权限不足、网络失败、编译失败等情况给出清晰错误提示并以非零状态退出。
- AC-6: 脚本支持重复执行时具备基本幂等性，不会因已存在目录、已安装文件而直接异常中断。
- AC-7: 脚本默认构建稳定 release 版本，而不是未固定版本的最新提交。

## Implementation Notes
- 建议脚本使用 `bash` 编写，并启用严格模式：`set -euo pipefail`。
- 构建方式可优先采用官方源码仓库 + 指定 tag 的方式，确保安装的是稳定 release。
- `fzf` 由 Go 编写，因此脚本需处理 Go 编译环境：
  - 若系统未安装 Go，可选择自动安装构建依赖；
  - 或明确要求用户预先安装满足版本要求的 Go。
- 安装流程建议包括：
  1. 检查 Ubuntu 环境；
  2. 检查并安装构建依赖（如 `git`, `curl`, `tar`, `make`, `golang`，具体以实际构建方式为准）；
  3. 拉取指定版本源码；
  4. 执行 release 构建；
  5. 将生成的 `fzf` 二进制复制到 `/usr/local/bin`；
  6. 将 shell 集成脚本安装到合适位置（如 `/usr/local/share/fzf` 或系统补全目录）；
  7. 为 bash/zsh 提供可选的 profile/source 提示。
- 若目标是“像 apt install 一样的效果”，需要注意：
  - `apt` 安装通常还会放置补全文件到系统目录；
  - 但不同 Ubuntu 版本和 `fzf` 包内容可能略有差异，因此应在脚本中明确模拟的范围。
- 建议支持通过变量配置：
  - `FZF_VERSION`：指定安装版本；
  - `PREFIX`：安装前缀，默认 `/usr/local`；
  - `INSTALL_SHELL_INTEGRATION=true|false`：是否安装 shell 集成。
- 建议使用 `sudo` 执行需要写入系统目录的步骤，或在脚本开头检查 root 权限。
- 如果采用 `git clone`，应使用浅克隆指定 tag，减少下载量。
- 如果采用 GitHub release 源码包而非完整仓库，可降低对 `git` 的依赖，但仍需确认 shell 集成资源包含在发布包中。
- 可在脚本末尾输出安装结果与后续提示，例如提示用户重新登录 shell 或手动 `source` 配置文件。

## Out of Scope
- 不包含为非 Ubuntu 发行版（如 Debian、CentOS、Arch）做兼容性适配。
- 不包含卸载脚本或完整包管理能力。
- 不包含制作 `.deb` 包。
- 不包含对 fish、PowerShell 等所有 shell 的全面集成，除非后续明确要求。
- 不包含长期维护自动升级逻辑。
- 不保证与 Ubuntu 官方仓库中 `fzf` 包的文件布局 100% 完全一致，只追求用户侧使用体验尽量接近。