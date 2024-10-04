go run *.go  -r clean -c  config.yml

go run *.go  -r 1 -c config.yml -m ii


 go run *.go  -r 1 -c config.yml -m ipgroup -delOldRule before
















go run *.go  -r 1 -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123



go run *.go  -r 1 -m ipgroup -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


go run *.go  -r 1 -m ispdomain -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


go run *.go  -r clean -c  /home/yh/workspace/ikuai-bypass/config_example.yml  -login http://10.1.1.1,admin,123

git tag -a v2.1.2-alpha1 -m "- 提供完整的最新的config.yml 文件，供参考
- 修复端口分流规则自动添加未能关联ip分组的bug，本次修改更新了一下config.yml的默认内容，请注意更新您的配置文件。[[#30]](https://github.com/joyanhui/ikuai-bypass/issues/30)
- 修复清理模式的删除规则问题 [[#27#issuecomment-2388114699]](https://github.com/joyanhui/ikuai-bypass/issues/27#issuecomment-2388114699)
- ip分组第一行的备注问题 [[#22]](https://github.com/joyanhui/ikuai-bypass/issues/22)
- 修复 卡`ip分组== 正在查询  备注为: IKUAI_BYPASS_ 的ip分组规则` 的bug  [[#24]](https://github.com/joyanhui/ikuai-bypass/issues/24) [[#27]](https://github.com/joyanhui/ikuai-bypass/issues/27)
- 修复运营商分流的ip列表会添加一个空行的bug [[#24]](https://github.com/joyanhui/ikuai-bypass/issues/24)"


git tag -a v2.0.1-alpha1 -m "增加删除旧规则的顺序的开关控制参数，此版本未经过测试，请谨慎使用"
git tag -a v2.0.0-beta2 -m "修复不添加-m参数无法执行的bug"
git tag -a v2.0.0-beta1 -m "增加ip分组和端口分流模式，增加命令行覆盖ikuai登陆参数模式。其他更新请参考readme或commit"





go run *.go  -r 1 -m ispdomain -delOldRule before  -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


git tag -d v2.0.0-beta2
git tag -d v2.0.0-beta1



git push origin --tags


git push origin :refs/tags/v1.0-test