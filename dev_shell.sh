go run *.go  -r 1 -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123



go run *.go  -r 1 -m ipgroup -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


go run *.go  -r 1 -m ispdomain -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


go run *.go  -r clean -c  /home/yh/workspace/ikuai-bypass/config_example.yml  -login http://10.1.1.1,admin,123

git tag -d v2.0.0-beta1

git tag -a v2.0.1-alpha1 -m "增加删除旧规则的顺序的开关控制参数，此版本未经过测试，请谨慎使用"
git tag -a v2.0.0-beta2 -m "修复不添加-m参数无法执行的bug"
git tag -a v2.0.0-beta1 -m "增加ip分组和端口分流模式，增加命令行覆盖ikuai登陆参数模式。其他更新请参考readme或commit"

go run *.go  -r 1 -m ispdomain -delOldRule before  -c  /home/yh/workspace/ikuai-bypass/config_example.yml -login http://10.1.1.1,admin,123


git tag -d v2.0.0-beta2
git tag -d v2.0.0-beta1

git push origin --tags


git push origin :refs/tags/v1.0-test