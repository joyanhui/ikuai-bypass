package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"strconv"
	"strings"
)

func (i *IKuai) ShowDomainGroupByTagName(tagName string) (result []ikuai_common.DomainGroupData, err error) {
	param := map[string]interface{}{
		"TYPE":    "total,data",
		"limit":   "0,1000",
		"FILTER1": "type,=,6", // Domain Group
	}
	req := CallReq{
		FuncName: FUNC_NAME_ROUTE_OBJECT,
		Action:   "show",
		Param:    &param,
	}

	var data4 []routeObject4
	resp := CallResp{Results: &CallRespData{Data: &data4}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Code != 0 {
		err = errors.New(resp.Message)
		return
	}

	for _, d := range data4 {
		if matchTagNameFilter(tagName, d.GroupName, d.Comment) {
			domains := make([]string, 0)
			for _, v := range d.GroupValue {
				if domain, ok := v["domain"]; ok {
					domains = append(domains, domain)
				}
			}
			result = append(result, ikuai_common.DomainGroupData{
				ID:        d.ID,
				GroupName: d.GroupName,
				Domains:   strings.Join(domains, ","),
				Comment:   d.Comment,
				Type:      d.Type,
			})
		}
	}
	return
}

func (i *IKuai) AddDomainGroup(groupName, domains string) error {
	domains = strings.TrimSpace(domains)
	domainList := strings.Split(domains, ",")
	groupValue := make([]map[string]string, 0)
	for _, domain := range domainList {
		domain = strings.TrimSpace(domain)
		if domain != "" {
			groupValue = append(groupValue, map[string]string{
				"domain":  domain,
				"comment": "",
			})
		}
	}

	param := map[string]interface{}{
		"group_name":  buildTagName(groupName),
		"type":        6, // Domain Group
		"group_value": groupValue,
		"comment":     "",
	}
	req := CallReq{
		FuncName: FUNC_NAME_ROUTE_OBJECT,
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

func (i *IKuai) DelDomainGroup(id string) error {
	param := struct {
		Id string `json:"id"`
	}{
		Id: id,
	}
	req := CallReq{
		FuncName: FUNC_NAME_ROUTE_OBJECT,
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

func (i *IKuai) GetDomainGroup(tag string) (preIds string, err error) {
	i.L.Info("QUERY:查询列表", "Querying domain group rules (Prefix: %s, Tag: %s)", ikuai_common.NAME_PREFIX_IKB, tag)
	var ids []string

	var data []ikuai_common.DomainGroupData
	data, err = i.ShowDomainGroupByTagName("")
	if err != nil {
		return "", err
	}

	for _, d := range data {
		if matchTagNameFilter(tag, d.GroupName, d.Comment) {
			ids = append(ids, strconv.Itoa(d.ID))
		}
	}

	if len(ids) <= 0 {
		return "", nil
	}

	preIds = strings.Join(ids, ",")

	return preIds, nil
}

func (i *IKuai) DelIKuaiBypassDomainGroup(cleanTag string) (err error) {
	for {
		var data []ikuai_common.DomainGroupData
		data, err = i.ShowDomainGroupByTagName("")
		if err != nil {
			return err
		}
		var ids []string
		for _, d := range data {
			if matchCleanTag(cleanTag, d.Comment, d.GroupName) {
				ids = append(ids, strconv.Itoa(d.ID))
			}
		}
		if len(ids) <= 0 {
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelDomainGroup(id)
		if err != nil {
			return
		}
	}
}
