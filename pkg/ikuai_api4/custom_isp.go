package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"
)

const FUNC_NAME_CUSTOM_ISP = "custom_isp"

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

func (i *IKuai) AddCustomIsp(name, tag, ipgroup string) error {
	// https://github.com/joyanhui/ikuai-bypass/issues/24
	// 去掉末尾空行、空格
	ipgroup = strings.TrimSpace(ipgroup)
	param := struct {
		Name    string `json:"name"`
		Ipgroup string `json:"ipgroup"`
		Comment string `json:"comment"`
	}{
		Name:    buildTagName(name),
		Ipgroup: ipgroup,
		Comment: "",
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

// GetCustomIspAll 预备删除
func (i *IKuai) GetCustomIspAll(tag string) (preIds string, err error) {
	log.Println("运营商/IP分流== 正在查询 名字前缀为:", ikuai_common.NAME_PREFIX_IKB, "且包含 tag:", tag, "的运营商配置规则")
	preIds = ""
	err = nil
	var data []ikuai_common.CustomIspData
	data, err = i.ShowCustomIspByTagName("")
	if err != nil {
		return
	}
	var ids []string
	for _, d := range data {
		if matchTagNameFilter(tag, d.Name, d.Comment) {
			ids = append(ids, strconv.Itoa(d.ID))
		}
	}
	if len(ids) <= 0 {
		return
	}

	id := strings.Join(ids, ",")
	preIds = preIds + "||" + id
	return
}

// DelCustomIspFromPreIds 删除
func (i *IKuai) DelCustomIspFromPreIds(preIds string) (err error) {
	arr := strings.Split(preIds, "||")
	for _, id := range arr {
		if len(id) < 1 {
			continue
		}
		err = i.DelCustomIsp(id)
		if err != nil {
			return
		}
	}
	return
}

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
