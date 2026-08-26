# 开发规范和项目概述
- AI agents may read and reference this file, but MUST NEVER modify it in any way.Provide suggestions only; all changes require manual human editing.
- 本文件为通用开发规范，项目特有内容见同目录 `@AGENTS_This.md`。 

## 语言和注释
- 沟通、文档使用中文；源码标识符、错误信息、接口返回、日志、git commit messages 使用英文（项目特殊约定见 `AGENTS_This.md`）。
- 除非明确要求，不得新增任何测试代码。
## Comments

- Do NOT add comments to code unless the user explicitly requests comments.
- Do NOT add explanatory, descriptive, or documentation comments on your own.
- Preserve all existing comments when modifying code.
- If a code change makes an existing comment factually incorrect or obsolete, update or remove that comment.
- Do not remove existing comments for cleanup, style, brevity, or readability reasons.
## 文档约定
- 文档只记录当前有效设计与约束，不写实现步骤、计划、历史沿革、历史对比、代码片段。
- 新建文档禁用 emoji 与 `**xx**`；多用 `-` 列表，避免多余空行与废话，不写大段示例代码和约定俗成文案。
- 新增核心功能必须同步更新对应模块文档；README 总览引用专题文档而非重复内容。
- 各项目文档目录、专题文档与事实来源见 `AGENTS_This.md`。

## 文件拆分与结构
- 单文件尽量 ≤800 行，>1000 行强制拆分；`lib.rs` / `mod.rs` / `index.ts` 只做导出，不写业务逻辑；按功能垂直拆分（`xx/xx.rs`）。
- 目录层级避免过深；同模块小文件用 `前缀+后缀.文件后缀` 前缀分组，文件过多再建子目录。
- 禁止用注解、注释、`_` 前缀或 eslint-disable 等绕过代码质量检查，必须彻底解决问题。

## Rust 规范
- 错误：生产代码全局禁用 `.unwrap()` / `.expect()`，用 `?` 或 `match`；错误码用统一枚举（唯一来源），API 层转前端友好格式，压平嵌套避免多层包装；
- 测试代码（`#[cfg(test)]` 模块与 `tests/` 集成测试）允许 `unwrap` / `expect`；生产代码由各 crate 根 `#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]` 强制拦截。
- 并发：全 `tokio` 异步；跨 `.await` 共享状态用 `RwLock`，独占状态用 `Mutex`；异步中禁止同步 IO / `thread::sleep` 等阻塞，CPU 密集用 `spawn_blocking`；`tokio::spawn` 必须带 `timeout`（≤ 24h）；捕获值须 `Send + 'static`，禁止 `Rc` / `RefCell` / 裸指针 / 持锁守卫传入；禁止持有 `MutexGuard` / `RwLockWriteGuard` 跨 `.await`；避免捕获局部借用的异步闭包。
- 后台任务须异常隔离，单任务 panic 不拖死全局循环，须有 supervisor 自愈。
- 所有异步任务必须具备明确的超时、取消和防重入机制：无论正常完成、超时或失败，占用的资源（连接、内存、信号量等）必须真正释放，任务可安全重试或跳过，不产生并发冲突与重复副作用。
- 优先借用、引用、迭代器链、模式匹配、`if let`、`let else`；`mut` 只在必要时使用；`.clone()` 是异味，优先借用 / 所有权转移 / `Cow` / `Arc`。
- 函数参数优先 `&str`、`&Path`、切片或泛型借用；仅在确需所有权、跨线程转移或持久化时用 `String` / `PathBuf`。
- 数据库用 ORM Query Builder，禁止 raw SQL（ORM 无法表达的 UNION 聚合等保留 parameterized 例外）；禁止 N+1；高频字段建索引；事务范围最小化，跨表修改必须同一事务完成，事务外不得有副作用写入。
- 数值运算用 `checked_add` / `checked_sub` / `saturating_mul` 等安全方式，禁止直接加减可能溢出的值。
- 禁止直接调用 `std::env::var` / `std::env::var_os`，统一走项目环境变量封装。
- 验收必须通过 `cargo clippy -- -D warnings` 并切实解决警告。但是除非明确要求，否则不要进行clippy验收和优化。

## 前端 / TypeScript 规范
- 技术栈：bun + React Router v7（React + TS + Vite）+ TanStack Query + Tailwind v4 + Zustand + shadcn/ui + Zod + React Hook Form + Lucide + Motion + typesafe-i18n + vite-plugin-checker + typescript-eslint。
- 类型安全：禁止 `any`；禁止 `@ts-ignore`、`as any`、非必要非空断言；导入类型用 `import type`；函数命名 camelCase，表/列命名 snake_case。
- 检查：必须通过 `tsc --noEmit`、`eslint`（0 错误 0 警告）、`vite build`；前端零 `any`、零 eslint-disable 注释、零 `_` 前缀规避。
- API 契约：响应统一 `ApiResponse`（`{ code, message, data }`）；错误码引用共享枚举，禁止硬编码数字；分页请求 `{ page, page_size }`、响应 `{ total, items }`；请求体解析必须校验并类型检查输入，禁止吞错误；API 类型复用自动生成的 OpenAPI / 错误码产物。
- UI：优先 shadcn/ui 组件 + Tailwind v4 原子类；移动优先（触控 ≥32px 按钮 / ≥24px 图标、字体 ≥14px、`Esc` 关闭弹层、避免 `hover` 作为唯一触发器）；主题三态由 CSS 变量 + ThemeProvider 驱动；全局样式放统一入口。
- 存储：统一 `@/lib/storage.ts` 管理；小数据 `localStorage`，大对象 IndexedDB；禁止组件直接 `setItem` / `getItem`。
- i18n：所有用户可见文本必须通过 typesafe-i18n（`LL.xxx()`），禁止硬编码文案；新增词条同步 `i18n-types.ts` 及所有语言 `index.ts`，嵌套结构一致（管理员页面除外）。
- 错误提示：后端只返回错误码与英文 message；前端按错误码映射 i18n 词条渲染，未覆盖码回退后端 message；网络/限流等通用错误映射约定词条；非组件模块通过 `getLL()` 获取当前语言 LL。
- Vite HMR：纯 CSS/TSX/TS 组件、非新增路由的改动绝不重启 bun dev 服务；只有 vite.config、新增路由、新增文件、env 变更等 HMR 覆盖不到的场景才允许重启。

## 错误码与 API 规范
- 错误码枚举为唯一来源：Rust 引用常量，TS 自动生成并 re-export，禁止硬编码数字。
- 所有 HTTP API 统一 `ApiResponse<T>` 返回；API 错误关联唯一错误码，API 层转前端友好 message，错误处理压平嵌套。
- 所有路由 RESTful + OpenAPI 3.0，operationId 全局唯一；OpenAPI 由 `#[utoipa::path]` 宏聚合到统一文档，新增接口必须登记进 `paths(...)`。
- 前端契约同步走统一入口（生成错误码 TS + OpenAPI 并转 TS 类型）；cargo run 调试启动自动同步（仅 debug + 开关允许 + 同进程一次，失败只记日志不阻断），Release 为空操作；CI 同步后校验产物与代码一致。

## 配置与环境变量规范
- 所有配置以项目配置文件为唯一真实来源（文件名与格式见 `AGENTS_This.md`）；禁止代码中存在配置未声明的配置项。
- 运行中可以变的配置项统一到数据库中，并合理利用缓存。
- 启动时找不到配置文件直接报错退出，禁止带病启动。
- 所有配置统一用 `*_env(key, default)` 读取，无启动时校验；env 不存在用代码默认值并打印警告；敏感配置（密码、密钥、token）必须随默认值打印警告。
- `string_env` 中 `""` 是合法值原样返回；`bool_env` / 数值中 `""` 回退 fallback；JSON 布尔值必须用 `true` / `false`，禁止字符串。
- 可选功能必须有独立 `ENABLED` 标志，禁止用 `""` 表示功能关闭。
- 环境变量带模块名前缀；同类配置用对象管理，变量使用相同前缀；简单结构写一行，嵌套不宜太深。
- 脚本禁止用 `env KEY=VALUE` 覆盖配置文件已加载的配置。
## 编译和构建
- 优先使用 base script/dev.sh <模块:动作  参数> （查看用法：bash script/dev.sh -h）
## 其他约定
- 可能存在用户或其他 agent 的未提交改动，禁止回滚、覆盖或整理与当前任务无关的改动。
- 为提高开发效率 尽量使用 `do sleep x` 避免使用  `timeout xx`执行命令
- 命令防卡死：`vite build`（vite-plugin-checker 构建完进程不退出）与 `typesafe-i18n`（默认 watch 永不退出，手工调用必须带 `--no-watch`）这类不退出进程不可接 `| tail` / `| head` 等等待管道 EOF 的命令；截断输出改用 `do sleep 5`或者 重定向到文件再 tail，如果非要用timeout 那么先执行依赖下载命令，timeout 的时间不要超过30秒。
- 禁止对 `bun run build`、`vite build`、`typesafe-i18n` 等不会自动退出的命令使用 `| head` / `| tail`。执行时应后台运行并重定向日志，使用 `do sleep 1` 后检查日志中的完成结果，不要等待进程退出,不要直接使用长时间sleep。
- GitHub Actions 记录用 `gh` 命令查看；Cloudflare Worker 交互用 `bunx wrangler`，明确由 GitHub 触发 Cloudflare Worker 构建的项目不手动执行 wrangler。
- 所有模块都需要带 /healthz /readyz 两个标准的api（不带/v1前缀）以兼容k8s的无版本号的路径

## 浏览器控制（Chrome DevTools MCP）
除非明确要求你使用浏览器调试，否则永远不要打开浏览器调试和截图。
需要控制浏览器时，必须使用 `chrome-devtools_*` MCP 工具（由 `chrome-devtools-mcp` 提供）。
- `chrome-devtools-mcp` 并没有调试和操作手机的tauri app的ui能力。要调试手机app请依赖adb。
- 禁止通过 bash 手动启动 Chrome（禁止 `google-chrome-stable --remote-debugging-port` 等命令）
- 除非远程调试手机的webdav，否则禁止使用 --headless（测试前端功能时 headless 会跳过 GPU 渲染、字体渲染、WebGL 等）
- 禁止使用 --user-data-dir=/tmp/...（临时目录）
- 禁止手动清理 user-data-dir 目录（MCP 服务器自动管理持久化 profile）
- 本机是nixos，chrome的路径是 /etc/profiles/per-user/y/bin/google-chrome-stable
