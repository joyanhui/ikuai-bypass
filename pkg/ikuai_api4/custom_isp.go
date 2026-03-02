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
		Name:    buildTagName(tag),
		Ipgroup: ipgroup,
		Comment: buildCustomIspChunkComment(index),
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
		Name:    buildTagName(tag),
		Ipgroup: ipgroup,
		Comment: buildCustomIspChunkComment(index),
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

	for _, d := range data {
		if matchTagNameFilter(tag, d.Name, d.Comment) {
			idx, ok := parseCustomIspChunkIndexFromComment(d.Comment)
			if !ok {
				idx, ok = parseCustomIspChunkIndexFromName(d.Name, tag)
			}
			if !ok {
				continue
			}
			if existedID, existed := result[idx]; existed {
				i.L.Warn("QUERY:序号冲突", "Duplicate custom ISP chunk %d for tag %s (keep ID: %d, skip ID: %d)", idx, tag, existedID, d.ID)
				continue
			}
			result[idx] = d.ID
		}
	}
	return result, nil
}

func buildCustomIspChunkComment(index int) string {
	chunk := index + 1
	if chunk <= 1 {
		return ikuai_common.NEW_COMMENT
	}
	return ikuai_common.NEW_COMMENT + "-" + strconv.Itoa(chunk)
}

func parseCustomIspChunkIndexFromComment(comment string) (int, bool) {
	comment = strings.TrimSpace(comment)
	if comment == "" {
		return 0, false
	}

	prefixes := []string{ikuai_common.NEW_COMMENT, ikuai_common.COMMENT_IKUAI_BYPASS}
	for _, prefix := range prefixes {
		if comment == prefix {
			return 1, true
		}
		if !strings.HasPrefix(comment, prefix) {
			continue
		}

		suffix := strings.TrimPrefix(comment, prefix)
		suffix = strings.TrimPrefix(suffix, "-")
		suffix = strings.TrimPrefix(suffix, "_")
		suffix = strings.TrimSpace(suffix)
		if suffix == "" {
			return 1, true
		}

		idx, err := strconv.Atoi(suffix)
		if err == nil && idx > 0 {
			return idx, true
		}
	}

	return 0, false
}

func parseCustomIspChunkIndexFromName(name, tag string) (int, bool) {
	baseName := buildTagName(tag)
	suffix := strings.TrimPrefix(strings.TrimSpace(name), baseName)
	if suffix == "" {
		return 0, false
	}
	idx, err := strconv.Atoi(suffix)
	if err != nil || idx <= 0 {
		return 0, false
	}
	return idx, true
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
