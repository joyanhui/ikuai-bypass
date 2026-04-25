增

{"action":"add","func_name":"route_object","param":{"group_name":"ipv6name","type":1,"group_value":[{"ipv6":"2e80::8252:aa1c:33ac:e8c9","comment":"备注1"},{"ipv6":"2e80::8252:aa1c:33ac:e8c9/64","comment":"备注2"}]}}
返回 
{"code":0,"message":"Success","rowid":3}

改

{"action":"edit","func_name":"route_object","param":{"group_name":"ipv6name22","type":1,"group_value":[{"ipv6":"2e80::8252:aa1c:33ac:e8c9","comment":"备注1"},{"ipv6":"2e80::8252:aa1c:33ac:e8c9/64","comment":"备注2"}],"id":3}}


删除
{"action":"del","func_name":"route_object","param":{"id":"3"}}

查
{"func_name":"route_object","action":"show","param":{"TYPE":"total,data","limit":"0,500","FILTER1":"type,=,1"}}
{
    "code": 0,
    "message": "Success",
    "results": {
        "total": 1,
        "data": [
            {
                "tagname": "ipv6name22",
                "group_id": "IPV6GP3",
                "group_value": [
                    {
                        "comment": "备注1",
                        "ipv6": "2e80::8252:aa1c:33ac:e8c9"
                    },
                    {
                        "comment": "备注2",
                        "ipv6": "2e80::8252:aa1c:33ac:e8c9\/64"
                    }
                ],
                "ref_count": 0,
                "id": 3,
                "group_name": "ipv6name22",
                "type": 1
            }
        ]
    }
}
