## iKuai Bypass

fork 自 https://github.com/ztc1997/ikuai-bypass/

## 主要修改点
- 并发处理 运营商/IP分流 和 域名分流  
- 更新成功后再删除旧规则  
- 支持无docker运行  
- 支持单次运行参数`-r nocron`忽略配置文件的cron配置
- 支持单独清理模式`-r clean` 清理本工具添加的备注为`IKUAI_BYPASS`的分流规则
- 支持cron运行参数`-r cron` `-r cronAft`
## 使用说明

[https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/](https://dev.leiyanhui.com/route/ikuai-bypass-joyanhui/)

## 配置文件参考

https://github.com/joyanhui/ikuai-bypass/blob/main/config_example.yml