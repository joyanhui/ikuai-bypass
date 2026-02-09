package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"
)

func (i *IKuai) ShowDomainGroupByComment(comment string) (result []ikuai_common.DomainGroupData, err error) {
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
		if comment == "" || d.Comment == comment || strings.Contains(d.Comment, comment) {
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
		"group_name":  NAME_PREFIX_IKB + groupName,
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
	log.Println("域名分组== 正在查询 名字前缀为:", NAME_PREFIX_IKB, "且包含 tag:", tag, "的域名分组规则")
	var ids []string

	var data []ikuai_common.DomainGroupData
	data, err = i.ShowDomainGroupByComment("")
	if err != nil {
		return "", err
	}

	for _, d := range data {
		if (strings.HasPrefix(d.GroupName, NAME_PREFIX_IKB) && strings.Contains(d.GroupName, tag)) || d.Comment == COMMENT_IKUAI_BYPASS+"_"+tag {
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
		data, err = i.ShowDomainGroupByComment("")
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
