package api

import (
	"errors"
	"log"
	"strconv"
	"strings"
)

const FUNC_NAME_CUSTOM_ISP = "custom_isp"

type CustomIspData struct {
	Ipgroup string `json:"ipgroup"`
	Time    string `json:"time"`
	ID      int    `json:"id"`
	Comment string `json:"comment"`
	Name    string `json:"name"`
}

func (i *IKuai) ShowCustomIspByComment() (result []CustomIspData, err error) {
	param := struct {
		Type    string `json:"TYPE"`
		Limit   string `json:"limit"`
		OrderBy string `json:"ORDER_BY"`
		Order   string `json:"ORDER"`
	}{
		Type: "data",
	}
	req := CallReq{
		FuncName: FUNC_NAME_CUSTOM_ISP,
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

func (i *IKuai) AddCustomIsp(name, tag, ipgroup string) error {
	param := struct {
		Name    string `json:"name"`
		Ipgroup string `json:"ipgroup"`
		Comment string `json:"comment"`
	}{
		Name:    name,
		Ipgroup: ipgroup,
		Comment: COMMENT_IKUAI_BYPASS + "_" + tag,
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
	if resp.Result != 30000 {
		return errors.New(resp.ErrMsg)
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
	if resp.Result != 30000 {
		return errors.New(resp.ErrMsg)
	}
	return nil
}

// PrepareDelCustomIspAll 预备删除
func (i *IKuai) PrepareDelCustomIspAll(tag string) (preIds string, err error) {
	log.Println("运营商/IP分流== 正在查询  备注为:", COMMENT_IKUAI_BYPASS+"_"+tag, "的运营商配置规则")
	preIds = ""
	err = nil
	//for loop := 0; loop < 3; loop++ {
	var data []CustomIspData
	data, err = i.ShowCustomIspByComment()
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
	preIds = preIds + "||" + id
	//fmt.Println("preIds", preIds)
	//err = i.DelCustomIsp(id)
	//if err != nil {
	//	return
	//}
	//}
	return
}

// 删除
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
		var data []CustomIspData
		data, err = i.ShowCustomIspByComment()
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
		err = i.DelCustomIsp(id)
		if err != nil {
			return
		}
	}
}
