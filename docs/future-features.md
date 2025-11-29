# 未来功能建议 (Future Feature Proposals)

本文档列出了 `prog` 项目管理工具未来可能实现的功能建议。这些功能基于开发者日常工作中的实际痛点，专注于解决磁盘空间管理、构建产物清理、依赖管理等实际问题。

---

## 1. 磁盘空间分析与清理 (`prog size` / `prog clean`)

**痛点**: 开发者管理大量仓库时，`node_modules`、`target`、`build` 等目录会占用大量磁盘空间。

### 1.1 磁盘占用分析 (`prog size`)

分析所有仓库的磁盘占用情况，特别关注构建产物和依赖目录：

```sh
> prog size
Scanning 42 repositories...

Total: 128.5 GB

Top 10 by size:
  1. ~/0Workspace/github.com/nicennnnnnnlee/test [4.2 GB]
     ├── node_modules/     2.1 GB
     ├── .next/            1.8 GB
     └── other             0.3 GB

  2. ~/0Workspace/github.com/nicennnnnnnlee/test2 [3.8 GB]
     ├── target/           3.2 GB (release: 1.8 GB, debug: 1.4 GB)
     └── other             0.6 GB

  3. ~/0Workspace/github.com/nicennnnnnnlee/test3 [2.9 GB]
     ├── node_modules/     2.5 GB
     └── dist/             0.4 GB

Cleanable space: 45.2 GB (node_modules: 28 GB, target: 12 GB, build: 5.2 GB)
```

**可选参数**:

```sh
> prog size --sort size          # 按大小排序（默认）
> prog size --sort name          # 按名称排序
> prog size --filter rust        # 只显示 Rust 项目
> prog size --filter node        # 只显示 Node.js 项目
> prog size --min 1GB            # 只显示大于 1GB 的仓库
> prog size prog                 # 分析特定仓库
```

### 1.2 构建产物清理 (`prog clean`)

智能清理各类项目的构建产物和缓存：

```sh
> prog clean --dry-run
Will clean:
  ~/0Workspace/github.com/nicennnnnnnlee/test/node_modules (2.1 GB)
  ~/0Workspace/github.com/nicennnnnnnlee/test/.next (1.8 GB)
  ~/0Workspace/github.com/nicennnnnnnlee/test2/target (3.2 GB)
  ...
Total: 45.2 GB

> prog clean --confirm
Cleaned 45.2 GB from 15 repositories.
```

**支持的清理目标**:

| 项目类型 | 清理目录 |
|---------|---------|
| Node.js | `node_modules/`, `dist/`, `.next/`, `.nuxt/`, `.output/` |
| Rust | `target/` |
| Go | `vendor/` (可选) |
| Python | `__pycache__/`, `.venv/`, `*.pyc` |
| Java/Kotlin | `build/`, `target/`, `.gradle/` |
| C/C++ | `build/`, `cmake-build-*/` |

**清理选项**:

```sh
> prog clean --all               # 清理所有可清理目录
> prog clean --node              # 只清理 node_modules
> prog clean --rust              # 只清理 Rust target
> prog clean --stale 30          # 只清理 30 天未修改的项目
> prog clean --keep-lock         # 保留 lock 文件，方便重新安装
> prog clean prog                # 只清理特定仓库
```

### 1.3 集成现有工具

考虑集成或调用现有的优秀工具来完成磁盘分析：

```sh
# 使用 dust 进行可视化分析
> prog size --use dust prog

# 使用 ncdu 进行交互式分析
> prog size --use ncdu prog

# 使用 dua 进行分析
> prog size --use dua prog
```

**配置默认工具**:

```toml
# ~/.prog/config.toml
[tools]
disk_analyzer = "dust"  # 可选: dust, ncdu, dua, builtin
```

---

## 2. 依赖管理 (`prog deps`)

**痛点**: 多个项目可能使用不同版本的依赖，想要统一管理或了解依赖状态。

### 2.1 依赖状态检查

```sh
> prog deps check
Checking dependencies in 42 repositories...

Outdated dependencies found:
  ~/0Workspace/github.com/nicennnnnnnlee/test
    └── package.json: 12 outdated (3 major, 5 minor, 4 patch)

  ~/0Workspace/github.com/nicennnnnnnlee/test2
    └── Cargo.toml: 5 outdated (1 major, 4 minor)

Security vulnerabilities:
  ~/0Workspace/github.com/nicennnnnnnlee/test
    └── lodash@4.17.15 - Prototype Pollution (HIGH)
```

### 2.2 批量更新依赖

```sh
> prog deps update --patch       # 只更新 patch 版本
> prog deps update --minor       # 更新 minor 版本
> prog deps update --dry-run     # 预览更新
```

---

## 3. 仓库状态总览 (`prog status`)

**痛点**: 管理大量仓库时，很难追踪哪些有未提交的更改或未推送的提交。

```sh
> prog status
Scanning 42 repositories...

⚠ Uncommitted changes (3):
  ~/0Workspace/github.com/nicennnnnnnlee/test [main]
    M src/index.ts
    A src/utils.ts

  ~/0Workspace/github.com/nicennnnnnnlee/test2 [feature/new]
    M lib.rs

  ~/0Workspace/github.com/nicennnnnnnlee/test3 [main]
    D old-file.js

↑ Unpushed commits (2):
  ~/0Workspace/github.com/nicennnnnnnlee/test4 [main] +3 commits
  ~/0Workspace/github.com/nicennnnnnnlee/test5 [develop] +1 commit

↓ Behind remote (1):
  ~/0Workspace/github.com/nicennnnnnnlee/test6 [main] -5 commits

✓ 36 repositories up to date
```

**选项**:

```sh
> prog status --dirty            # 只显示有未提交更改的仓库
> prog status --unpushed         # 只显示有未推送提交的仓库
> prog status --behind           # 只显示落后于远程的仓库
```

---

## 4. 批量 Git 操作 (`prog batch`)

**痛点**: 需要对多个仓库执行相同的 Git 操作。

### 4.1 批量拉取

```sh
> prog batch pull
Pulling 42 repositories (4 parallel jobs)...
[████████████████████████████] 100% (42/42)

Results:
  ✓ 38 updated successfully
  ⚠ 2 had conflicts
  ✗ 2 failed (network error)
```

### 4.2 批量执行命令

```sh
> prog batch exec "git fetch --prune"
> prog batch exec "npm install" --filter node
> prog batch exec "cargo update" --filter rust
```

---

## 5. 快速导航增强

### 5.1 最近访问 (`prog recent`)

```sh
> prog recent
1. [2 min ago]  ~/0Workspace/github.com/bytemain/prog
2. [1 hour ago] ~/0Workspace/github.com/microsoft/vscode
3. [3 hours ago] ~/0Workspace/github.com/nicennnnnnnlee/test

> prog recent 2  # 直接跳转到第 2 个
```

### 5.2 编辑器集成 (`prog open`)

```sh
> prog open prog                # 用默认编辑器打开
> prog open prog --code         # 用 VS Code 打开
> prog open prog --idea         # 用 IntelliJ IDEA 打开
> prog open prog --cursor       # 用 Cursor 打开
```

**配置**:

```toml
# ~/.prog/config.toml
[editor]
default = "code"
# 或使用自定义命令
default = "cursor --new-window"
```

### 5.3 在浏览器中打开 (`prog browse`)

```sh
> prog browse prog              # 打开仓库主页
> prog browse prog -i           # 打开 Issues 页面
> prog browse prog -p           # 打开 Pull Requests 页面
```

---

## 6. 项目健康检查 (`prog doctor`)

**痛点**: 有时仓库会出现各种问题，需要诊断和修复。

```sh
> prog doctor
Running health checks on 42 repositories...

Issues found:
  ~/0Workspace/github.com/nicennnnnnnlee/test
    ⚠ Large files in history (> 100MB)
    ⚠ node_modules committed to repo
    
  ~/0Workspace/github.com/nicennnnnnnlee/test2
    ✗ Corrupted git index
    ⚠ Remote 'origin' not accessible
    
  ~/0Workspace/github.com/nicennnnnnnlee/test3
    ⚠ Detached HEAD state
    ⚠ Stale branches (5 merged branches)

Suggestions:
  Run 'prog doctor --fix' to auto-fix recoverable issues
  Run 'prog gc' to clean up git objects
```

**检查项目**:
- Git 索引完整性
- 远程连接状态
- 大文件检测
- 分离的 HEAD 状态
- 过期的本地分支
- `.gitignore` 配置检查

---

## 7. Git 仓库优化 (`prog gc`)

**痛点**: Git 仓库随着时间推移会积累大量对象，影响性能。

```sh
> prog gc
Running garbage collection on 42 repositories...

Optimized:
  ~/0Workspace/github.com/nicennnnnnnlee/test: 150 MB → 80 MB (-70 MB)
  ~/0Workspace/github.com/nicennnnnnnlee/test2: 89 MB → 45 MB (-44 MB)

Total space recovered: 1.2 GB
```

**选项**:

```sh
> prog gc --aggressive          # 更彻底的清理（耗时更长）
> prog gc --prune               # 同时清理过期的远程追踪分支
```

---

## 实现优先级建议

### 高优先级（解决实际痛点）

1. **`prog size`** - 磁盘占用分析（集成 dust 等工具）
2. **`prog clean`** - 构建产物清理（node_modules, target 等）
3. **`prog status`** - 仓库状态总览
4. **`prog open`** - 编辑器集成

### 中优先级（提升效率）

5. **`prog batch pull`** - 批量拉取更新
6. **`prog recent`** - 最近访问历史
7. **`prog browse`** - 浏览器快速打开
8. **`prog gc`** - Git 仓库优化

### 低优先级（高级功能）

9. **`prog deps`** - 依赖管理
10. **`prog doctor`** - 健康检查
11. **`prog batch exec`** - 批量执行命令

---

## 外部工具推荐

在实现这些功能之前，可以考虑集成以下现有的优秀工具：

| 工具 | 用途 | 安装方式 |
|-----|------|---------|
| [dust](https://github.com/bootandy/dust) | 磁盘空间可视化 | `cargo install du-dust` |
| [ncdu](https://dev.yorhel.nl/ncdu) | 交互式磁盘分析 | `apt install ncdu` |
| [dua](https://github.com/Byron/dua-cli) | 快速磁盘分析 | `cargo install dua-cli` |
| [npkill](https://github.com/voidcosmos/npkill) | 清理 node_modules | `npx npkill` |
| [cargo-sweep](https://github.com/holmgr/cargo-sweep) | 清理 Rust target | `cargo install cargo-sweep` |
| [git-sizer](https://github.com/github/git-sizer) | 分析 Git 仓库大小 | `brew install git-sizer` |

---

## 结语

以上功能建议基于开发者日常工作中的实际需求。优先实现磁盘空间管理相关功能，因为这是管理大量仓库时最常见的痛点。

如有问题或建议，欢迎提交 Issue 或 Pull Request。
