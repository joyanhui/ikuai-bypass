package api

import (
	"errors"
	"log"
	"strconv"
	"strings"
)

const FUNC_NAME_IP_GROUP = "ipgroup"

type IpGroupData struct {
	AddrPool  string `json:"addr_pool"`
	Comment   string `json:"comment"`
	GroupName string `json:"group_name"`
	ID        int    `json:"id"`
	Type      int    `json:"type"`
}

func (i *IKuai) ShowIpGroupByComment(comment string) (result []IpGroupData, err error) {
	param := struct {
		Finds    string `json:"FINDS"`
		Keywords string `json:"KEYWORDS"`
		Type     string `json:"TYPE"`
		Limit    string `json:"limit"`
		OrderBy  string `json:"ORDER_BY"`
		Order    string `json:"ORDER"`
	}{
		Finds:    "comment",
		Keywords: comment,
		Type:     "data",
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
		Action:   "show",
		Param:    &param,
	}
	resp := CallResp{Data: &CallRespData{Data: &result}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Result != 30000 {
		err = errors.New(resp.ErrMsg)
		return
	}
	return
}

func (i *IKuai) ShowIpGroupByName(name string) (result []IpGroupData, err error) {
	param := struct {
		Finds    string `json:"FINDS"`
		Keywords string `json:"KEYWORDS"`
		Type     string `json:"TYPE"`
		Limit    string `json:"limit"`
		OrderBy  string `json:"ORDER_BY"`
		Order    string `json:"ORDER"`
	}{
		Finds:    "group_name",
		Keywords: name,
		Type:     "data",
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
		Action:   "show",
		Param:    &param,
	}
	resp := CallResp{Data: &CallRespData{Data: &result}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Result != 30000 {
		err = errors.New(resp.ErrMsg)
		return
	}
	return
}

func (i *IKuai) AddIpGroup(groupName, addrPool string) error {
	param := struct {
		AddrPool  string `json:"addr_pool"`
		Comment   string `json:"comment"`
		GroupName string `json:"group_name"`
		NewRow    bool   `json:"newRow"`
		Type      int    `json:"type"`
	}{
		GroupName: groupName,
		AddrPool:  addrPool,
		Comment:   COMMENT_IKUAI_BYPASS + "_" + groupName, //自定义的备注无效的问题
		NewRow:    true,
		Type:      0,
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
		Action:   "add",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Result != 30000 {
		return errors.New(resp.ErrMsg)
	}
	return nil
}

func (i *IKuai) DelIpGroup(id string) error {
	param := struct {
		Id string `json:"id"`
	}{
		Id: id,
	}
	req := CallReq{
		FuncName: FUNC_NAME_IP_GROUP,
		Action:   "del",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Result != 30000 {
		return errors.New(resp.ErrMsg)
	}
	return nil
}

func (i *IKuai) GetIpGroup(tag string) (preIds string, err error) {
	log.Println("ip分组== 正在查询  备注为:", COMMENT_IKUAI_BYPASS+"_"+tag, "的ip分组规则")
	var tagComment = ""
	if tag == "" {
		tagComment = COMMENT_IKUAI_BYPASS
	} else {
		tagComment = COMMENT_IKUAI_BYPASS + "_" + tag
	}

	var ids []string // 初始化 ids 切片

	var data []IpGroupData
	data, err = i.ShowIpGroupByComment(tagComment)  // 获取数据并处理错误
	if err != nil {
		return "", err // 返回错误
	}

	for _, d := range data {
		ids = append(ids, strconv.Itoa(d.ID))
	}

        // 如果没有找到匹配的IP分组，则返回空字符串和nil error
	if len(ids) <= 0 {
		return "", nil // 返回空字符串和 nil 错误
	}

	preIds = strings.Join(ids, ",")  // 将 IDs 连接成逗号分隔的字符串
	log.Println("ip分组== 旧的id为：", preIds)

	return preIds, nil   // 返回 IDs 和 nil 错误
}

func (i *IKuai) DelIKuaiBypassIpGroup(cleanTag string) (err error) {

	for {
		var data []IpGroupData
		data, err = i.ShowIpGroupByComment(COMMENT_IKUAI_BYPASS)
		var ids []string
		for _, d := range data {
			//log.Println("在判断:", d.GroupName, d.Comment)
			if cleanTag == "cleanAll" {
				if d.Comment == COMMENT_IKUAI_BYPASS || strings.Contains(d.Comment, COMMENT_IKUAI_BYPASS) {
					ids = append(ids, strconv.Itoa(d.ID))
				}
			} else {
				if cleanTag == "" {
					cleanTag = COMMENT_IKUAI_BYPASS
				}
				if d.Comment == cleanTag || d.Comment == COMMENT_IKUAI_BYPASS+"_"+cleanTag {
					ids = append(ids, strconv.Itoa(d.ID))
				}
			}
		}
		if len(ids) <= 0 {
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelIpGroup(id)
		if err != nil {
			return
		}
	}
}

func (i *IKuai) GetAllIKuaiBypassIpGroupNamesByName(name string) (names []string, err error) {
	var data []IpGroupData
	data, err = i.ShowIpGroupByName(name)

	for _, d := range data {
		// for https://github.com/joyanhui/ikuai-bypass/issues/30
		// fix 前面修改ip分组的备注导致的 无法甄别ip分组的问题
		//match, _ := regexp.MatchString(name+`_\d+`, d.GroupName)
		//log.Println(d.GroupName)
		match := strings.Contains(d.GroupName, name)
		if (d.Comment == COMMENT_IKUAI_BYPASS || strings.Contains(d.Comment, COMMENT_IKUAI_BYPASS)) && match {
			names = append(names, d.GroupName)
		}
	}
	return
}
