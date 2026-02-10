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
	// https://github.com/joyanhui/ikuai-bypass/issues/24
	// 去掉末尾空行、空格
	// Remove trailing empty lines and spaces
	ipgroup = strings.TrimSpace(ipgroup)

	// 自定义运营商支持同名，这里统一使用 buildTagName，并在备注中区分编号
	// Custom ISP supports duplicate names. Use buildTagName and distinguish by index in comment.
	comment := ikuai_common.NEW_COMMENT
	if index > 0 {
		comment += "-" + strconv.Itoa(index+1)
	}

	param := struct {
		Name    string `json:"name"`
		Ipgroup string `json:"ipgroup"`
		Comment string `json:"comment"`
	}{
		Name:    buildTagName(tag),
		Ipgroup: ipgroup,
		Comment: comment,
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

// GetCustomIspAll 准备查询需要删除的旧规则 ID
// GetCustomIspAll prepares the IDs of old rules to be deleted
func (i *IKuai) GetCustomIspAll(tag string) (preIds string, err error) {
	i.L.Info("QUERY:查询规则", "Querying custom ISP rules (Prefix: %s, Tag: %s)", ikuai_common.NAME_PREFIX_IKB, tag)
	preIds = ""
	err = nil
	var data []ikuai_common.CustomIspData
	data, err = i.ShowCustomIspByTagName("")
	if err != nil {
		return
	}
	var ids []string
	for _, d := range data {
		// 优先名字匹配，兼容序号
		// Priority matching by name, compatible with sequence numbers
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

// DelCustomIspFromPreIds 根据提供的 ID 列表删除旧规则
// DelCustomIspFromPreIds deletes old rules based on the provided ID list
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
