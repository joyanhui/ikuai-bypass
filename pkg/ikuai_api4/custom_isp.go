package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"strconv"
	"strings"
)

const FUNC_NAME_CUSTOM_ISP = "custom_isp"

// ShowCustomIspByTagName 根据标签名称查询自定义运营商规则
// ShowCustomIspByTagName queries custom ISP rules by tag name
func (i *IKuai) ShowCustomIspByTagName(tagName string) (result []ikuai_common.CustomIspData, err error) {
	param := struct {
		Type    string `json:"TYPE"`
		Limit   string `json:"limit"`
		OrderBy string `json:"ORDER_BY"`
		Order   string `json:"ORDER"`
	}{
		Type:  "total,data",
		Limit: "0,1000",
	}
	req := CallReq{
		FuncName: FUNC_NAME_CUSTOM_ISP,
		Action:   "show",
		Param:    &param,
	}
	var all []ikuai_common.CustomIspData
	resp := CallResp{Results: &CallRespData{Data: &all}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Code != 0 {
		err = errors.New(resp.Message)
		return
	}
	for _, d := range all {
		if matchTagNameFilter(tagName, d.Name, d.Comment) {
			result = append(result, d)
		}
	}
	return
}

// AddCustomIsp 添加自定义运营商规则
// AddCustomIsp adds a custom ISP rule
func (i *IKuai) AddCustomIsp(tag, ipgroup string, index int) error {
	ipgroup = strings.TrimSpace(ipgroup)

	param := struct {
		Name    string `json:"name"`
		Ipgroup string `json:"ipgroup"`
		Comment string `json:"comment"`
	}{
		Name:    buildIndexedTagName(tag, index),
		Ipgroup: ipgroup,
		Comment: ikuai_common.NEW_COMMENT,
	}
	req := CallReq{
		FuncName: FUNC_NAME_CUSTOM_ISP,
		Action:   "add",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}

func (i *IKuai) EditCustomIsp(tag, ipgroup string, index int, id int) error {
	ipgroup = strings.TrimSpace(ipgroup)

	param := struct {
		Name    string `json:"name"`
		Ipgroup string `json:"ipgroup"`
		Comment string `json:"comment"`
		ID      int    `json:"id"`
	}{
		Name:    buildIndexedTagName(tag, index),
		Ipgroup: ipgroup,
		Comment: ikuai_common.NEW_COMMENT,
		ID:      id,
	}
	req := CallReq{
		FuncName: FUNC_NAME_CUSTOM_ISP,
		Action:   "edit",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}

func (i *IKuai) GetCustomIspMap(tag string) (result map[int]int, err error) {
	result = make(map[int]int)
	var data []ikuai_common.CustomIspData
	data, err = i.ShowCustomIspByTagName("")
	if err != nil {
		return nil, err
	}

	baseName := buildTagName(tag)
	for _, d := range data {
		if matchTagNameFilter(tag, d.Name, d.Comment) {
			suffix := strings.TrimPrefix(d.Name, baseName)
			if idx, err := strconv.Atoi(suffix); err == nil {
				result[idx] = d.ID
			}
		}
	}
	return result, nil
}

// DelCustomIsp 删除自定义运营商规则
// DelCustomIsp deletes a custom ISP rule
func (i *IKuai) DelCustomIsp(id string) error {
	param := struct {
		Id string `json:"id"`
	}{
		Id: id,
	}
	req := CallReq{
		FuncName: FUNC_NAME_CUSTOM_ISP,
		Action:   "del",
		Param:    &param,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return err
	}
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}


// DelCustomIspAll 清理所有符合条件的自定义运营商规则
// DelCustomIspAll cleans up all custom ISP rules that match the condition
func (i *IKuai) DelCustomIspAll(cleanTag string) (err error) {
	for {
		var data []ikuai_common.CustomIspData
		data, err = i.ShowCustomIspByTagName("")
		if err != nil {
			return
		}
		var ids []string
		for _, d := range data {
			if matchCleanTag(cleanTag, d.Comment, d.Name) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) <= 0 {
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelCustomIsp(id)
		if err != nil {
			return
		}
	}
}
