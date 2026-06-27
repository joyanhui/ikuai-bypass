增
{"action":"add","func_name":"route_object","param":{"group_name":"ipv4分组名字","type":0,"group_value":[{"ip":"192.168.1.0/24","comment":""},{"ip":"192.168.1.1-192.168.1.111","comment":"备注3"}]}}

{"action":"add","func_name":"route_object","param":{"group_name":"ipv6name","type":1,"group_value":[{"ipv6":"2e80::8252:aa1c:33ac:e8c9","comment":"备注1"},{"ipv6":"2e80::8252:aa1c:33ac:e8c9/64","comment":"备注2"}]}}
返回 
{"code":0,"message":"Success","rowid":3}

改
{"action":"edit","func_name":"route_object","param":{"group_name":"ipv4分组名字","type":0,"group_value":[{"ip":"192.168.1.0/24","comment":""},{"ip":"192.168.1.1-192.168.1.111","comment":"备注4"}],"id":3}}

{"action":"edit","func_name":"route_object","param":{"group_name":"ipv6name22","type":1,"group_value":[{"ipv6":"2e80::8252:aa1c:33ac:e8c9","comment":"备注1"},{"ipv6":"2e80::8252:aa1c:33ac:e8c9/64","comment":"备注2"}],"id":3}}


删除
{"action":"del","func_name":"route_object","param":{"id":"3,4"}}
查
{"func_name":"route_object","action":"show","param":{"TYPE":"total,data","limit":"0,500","FILTER1":"type,=,1"}}

{"func_name":"route_object","action":"show","param":{"TYPE":"total,data","limit":"0,500","FILTER1":"type,=,0"}}

{
    "code": 0,
    "message": "Success",
    "results": {
        "total": 2,
        "data": [
            {
                "id": 3,
                "group_name": "ipv4分组名字",
                "ref_count": 0,
                "tagname": "ipv4分组名字",
                "group_id": "IPGP3",
                "type": 0,
                "group_value": [
                    {
                        "comment": "",
                        "ip": "192.168.1.0\/24"
                    },
                    {
                        "comment": "备注4",
                        "ip": "192.168.1.1-192.168.1.111"
                    }
                ]
            },
            {
                "id": 4,
                "group_name": "ipv4_name1",
                "ref_count": 0,
                "tagname": "ipv4_name1",
                "group_id": "IPGP4",
                "type": 0,
                "group_value": [
                    {
                        "comment": "备注1",
                        "ip": "92.168.1.1"
                    },
                    {
                        "comment": "",
                        "ip": "192.168.1.0\/24"
                    }
                ]
            }
        ]
    }
}
