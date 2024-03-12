##  简述
这是只是一个思路和演示，单网卡这样搞性能并不好。很多因素会导致跑不满理论带宽。    
- windows 宿主机 + VMware  单物理网卡
- 交换机
- 光猫
- AP若干
- 其他设备

openwrt 部署在爱快的vm内
## 物理连接
所有设备一股脑都插到交换机

## ikuai网卡说明
- lan1 使用VMware NAT网卡 vmnet8  仅供 windows宿主机管理爱快     
- wan1 桥接物理网卡，也就是vmnet0  作为pppoe 通过交换机再到光猫拨号    
- lan2 桥接物理网卡，也就是vmnet0   通过交换机给局域网内其他设备提供上网服务和dhcp   
- wan2 使用VMware NAT网卡 vmnet10 ，VMware默认没启用自己启用一下， 作为openwrt下游   

## ikuai内-vm-openwrt网卡说明
- lan 桥接 爱快的wan2 ，为局域网提供代理功能，作为爱快wan2的上游    
- wan 桥接 爱快的lan1 ，作为ikuai的下的一个设备，openwrt自己连接互联网用   

## 关于vmnet10
这里的vmnet10 单独给爱快的openwrt用的。VMware默认也没有启用这个网卡，实际上也非必须，可以用vmnet8替代。不过建议还是启用一下，方便隔离和管理。
