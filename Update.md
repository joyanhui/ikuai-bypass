 v4.4.4-Pre (内测版本)
> 因爱快v4 新发布且处于内测阶段，API改动较大，此版本ikuai-bypass仅作尝鲜体验

### 1. 完成爱快 v4 内测版适配 [#103]
- 适配 iKuai8_x64_4.0.0_Build202512241218
- [重要] 统一使用 v4 API，移除 ikuai_api3 支持
- [重要] v4.4 以后不再支持爱快 v3.x 版本（v3.7 及以下请用 v4.1）
- [重要] 规则标识从备注改为名字前缀 `IKB`（v4 接口备注无返回）
- [重要] 配置项统一使用 `tag`，删除 `name` 等混乱字段，参考 [config.yml](https://github.com/joyanhui/ikuai-bypass/blob/main/config.yml) 更新配置

### 2. 新增 iip 混合模式 [#104]
- 支持 `ispgroup`、`ipv4group`、`ipv6group` 三分流模块一起使用，参数：`-m iip`

### 3. 其他优化
- 去除并发处理，改为顺序执行，避免 API 失败和日志乱序
- 重构日志系统，增加智能高亮、彩色输出和跨平台支持
- [重要] 因 v4 API 限制同名规则，移除 `delOldRule` 参数，统一为先同步后查询再删除
- 删除 `exportDomainSteamToTxt` 功能
- [重要] 清理模式必须显式配置 `-tag` 参数，不再默认 `CleanAll`，仅清理名字包含 `IKB` 开头的规则（兼容旧版备注含 `IKUAI_BYPASS` 或 `joyanhui/ikuai-bypass` 的规则）
- ip分组去掉注释功能 因为爱快v4的ip分组功能的注释实际上是行注释已经没有原来的作用。
- 修复 v4.4.2-Pre 自定义运营商的 bug [#105](https://github.com/joyanhui/ikuai-bypass/issues/105#issuecomment-3875268800)
- 增加配置 可以控制 单条条分组/分流规则的 最大记录数量，可以绕开爱快的限制，分别增加50%或者数倍. 
    - 参考[#105](https://github.com/joyanhui/ikuai-bypass/issues/105#issuecomment-3875268800)
    - 参考[config.yml#L67](https://github.com/joyanhui/ikuai-bypass/blob/9146f7581eb6a549546b6a241df79b0b34742f83/config.yml#L67)
