package api

import (
	"crypto/md5"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"log"
	"net/http"
	"net/http/cookiejar"
	"time"
)

const COMMENT_IKUAI_BYPASS = "IKUAI_BYPASS"

type IKuai struct {
	baseurl string
	client  *http.Client
}

type CallReq struct {
	FuncName string      `json:"func_name"`
	Action   string      `json:"action"`
	Param    interface{} `json:"param"`
}

type CallResp struct {
	ErrMsg string        `json:"ErrMsg"`
	Result int           `json:"Result"`
	RowID  int           `json:"RowId"`
	Data   *CallRespData `json:"Data"`
}

type CallRespData struct {
	Total int         `json:"total"`
	Data  interface{} `json:"data"`
}

func md5String(v string) string {
	d := []byte(v)
	m := md5.New()
	m.Write(d)
	return hex.EncodeToString(m.Sum(nil))
}

func NewIKuai(baseurl string) *IKuai {
	cookieJar, err := cookiejar.New(nil)
	if err != nil {
		log.Fatalln(err)
	}
	return &IKuai{baseurl, &http.Client{Jar: cookieJar, Timeout: time.Second * 10}}
}

func (i *IKuai) Login(username, password string) error {
	passwd := md5String(password)
	pass := base64.StdEncoding.EncodeToString([]byte("salt_11" + password))
	req := map[string]string{
		"passwd":            passwd,
		"pass":              pass,
		"remember_password": "",
		"username":          username,
	}
	resp := CallResp{}
	err := postJson(i.client, i.baseurl+"/Action/login", req, &resp)
	if err != nil {
		return err
	}
	if resp.Result != 10000 {
		return errors.New(resp.ErrMsg)
	}
	return nil
}
