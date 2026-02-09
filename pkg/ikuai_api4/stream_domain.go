package ikuai_api4

import (
	"errors"
	"ikuai-bypass/pkg/ikuai_common"
	"log"
	"strconv"
	"strings"
)

const FuncNameStreamDomain = "stream_domain"

// 4.0 域名分流专用结构
// Specific structures for 4.0 stream_domain
type streamDomain4 struct {
	ID        int    `json:"id"`
	Enabled   string `json:"enabled"`
	Tagname   string `json:"tagname"`
	Interface string `json:"interface"`
	Comment   string `json:"comment"`
	Prio      int    `json:"prio"`
	SrcAddr   struct {
		Custom []string    `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"src_addr"`
	Domain struct {
		Custom []string    `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"domain"`
	Time struct {
		Custom []struct {
			Type      string `json:"type"`
			Weekdays  string `json:"weekdays"`
			StartTime string `json:"start_time"`
			EndTime   string `json:"end_time"`
			Comment   string `json:"comment"`
		} `json:"custom"`
		Object interface{} `json:"object"`
	} `json:"time"`
}

func (i *IKuai) AddStreamDomain(iface, tag, srcAddr, domains string, index int) error {
	// https://github.com/joyanhui/ikuai-bypass/issues/24
	// 去掉末尾空行、空格
	domains = strings.TrimSpace(domains)
	domainList := strings.Split(domains, ",")

	srcAddr = strings.TrimSpace(srcAddr)
	var srcAddrList []string
	if srcAddr != "" {
		srcAddrList = strings.Split(srcAddr, ",")
	} else {
		srcAddrList = []string{}
	}

	// 使用序号作为 tagname 后缀，从 1 开始，防止 chunks 冲突
	// Use sequence number starting from 1 as tagname suffix to avoid chunks conflicts
	uniqueTagname := tag + "_" + strconv.Itoa(index+1)

	// 构造 4.0 格式的参数
	// Construct 4.0 format parameters
	param := map[string]interface{}{
		"enabled":   "yes",
		"tagname":   uniqueTagname,
		"interface": iface,
		"src_addr": map[string]interface{}{
			"custom": srcAddrList,
			"object": []interface{}{},
		},
		"domain": map[string]interface{}{
			"custom": domainList,
			"object": []interface{}{},
		},
		"comment": COMMENT_IKUAI_BYPASS + "_" + tag,
		"time": map[string]interface{}{
			"custom": []map[string]interface{}{
				{
					"type":       "weekly",
					"weekdays":   "1234567",
					"start_time": "00:00",
					"end_time":   "23:59",
					"comment":    "",
				},
			},
			"object": []interface{}{},
		},
		"prio": 31,
	}

	req := CallReq{
		FuncName: FuncNameStreamDomain,
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

func (i *IKuai) ShowStreamDomainByComment(comment string) (result []ikuai_common.StreamDomainData, err error) {
	param := struct {
		Type     string `json:"TYPE"`
		Limit    string `json:"limit"`
		OrderBy  string `json:"ORDER_BY"`
		Order    string `json:"ORDER"`
		Finds    string `json:"FINDS"`
		Keywords string `json:"KEYWORDS"`
	}{
		Finds:    "comment",
		Keywords: comment,
		Type:     "total,data",
		Limit:    "0,1000",
	}
	req := CallReq{
		FuncName: FuncNameStreamDomain,
		Action:   "show",
		Param:    &param,
	}

	var data4 []streamDomain4
	resp := CallResp{Results: &CallRespData{Data: &data4}}
	err = postJson(i.client, i.baseurl+"/Action/call", &req, &resp)
	if err != nil {
		return
	}
	if resp.Code != 0 {
		err = errors.New(resp.Message)
		return
	}

	// 将 4.0 结构转换为通用结构
	// Convert 4.0 structure to common structure
	result = make([]ikuai_common.StreamDomainData, len(data4))
	for idx, d := range data4 {
		result[idx] = ikuai_common.StreamDomainData{
			ID:        d.ID,
			Enabled:   d.Enabled,
			Comment:   d.Comment,
			Interface: d.Interface,
			Domain:    strings.Join(d.Domain.Custom, ","),
			SrcAddr:   strings.Join(d.SrcAddr.Custom, ","),
		}
		// 尝试还原 Week 和 Time 字段
		if len(d.Time.Custom) > 0 {
			result[idx].Week = d.Time.Custom[0].Weekdays
			result[idx].Time = d.Time.Custom[0].StartTime + "-" + d.Time.Custom[0].EndTime
		}
	}

	return
}

func (i *IKuai) DelStreamDomain(id string) error {
	param := struct {
		Id string `json:"id"`
	}{
		Id: id,
	}
	req := CallReq{
		FuncName: FuncNameStreamDomain,
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

// GetStreamDomainAll 批量查询并返回逗号分隔的 ID
// Batch query and return comma-separated IDs
func (i *IKuai) GetStreamDomainAll(tag string) (preIds string, err error) {
	log.Println("域名分流== 正在查询  备注为:", COMMENT_IKUAI_BYPASS+"_"+tag, "的域名分流规则")
	preIds = ""
	err = nil
	var data []ikuai_common.StreamDomainData
	data, err = i.ShowStreamDomainByComment(COMMENT_IKUAI_BYPASS + "_" + tag)
	if err != nil {
		return
	}
	var ids []string
	for _, d := range data {
		if d.Comment == COMMENT_IKUAI_BYPASS+"_"+tag {
			ids = append(ids, strconv.Itoa(d.ID))
		}
	}
	if len(ids) <= 0 {
		return
	}
	id := strings.Join(ids, ",")
	preIds = "||" + id
	return
}

// DelStreamDomainFromPreIds 从预备删除的id中删除
func (i *IKuai) DelStreamDomainFromPreIds(preIds string) (err error) {
	arr := strings.Split(preIds, "||")
	for _, id := range arr {
		if len(id) < 1 {
			continue
		}
		err = i.DelStreamDomain(id)
		if err != nil {
			return
		}
	}
	return

}

// DelStreamDomainAll 删除所有的域名分流规则
func (i *IKuai) DelStreamDomainAll(cleanTag string) (err error) {
	for {
		var data []ikuai_common.StreamDomainData
		data, err = i.ShowStreamDomainByComment(COMMENT_IKUAI_BYPASS)
		if err != nil {
			return
		}
		var ids []string
		for _, d := range data {
			if cleanTag == "cleanAll" {
				if d.Comment == COMMENT_IKUAI_BYPASS || strings.Contains(d.Comment, COMMENT_IKUAI_BYPASS) {
					ids = append(ids, strconv.Itoa(d.ID))
				}
			} else {
				if d.Comment == cleanTag {
					ids = append(ids, strconv.Itoa(d.ID))
				}
			}
		}
		if len(ids) <= 0 {
			return
		}
		id := strings.Join(ids, ",")
		err = i.DelStreamDomain(id)
		if err != nil {
			return
		}
	}
}