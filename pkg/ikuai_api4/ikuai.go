package ikuai_api4

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

const FUNC_NAME_ROUTE_OBJECT = "route_object"

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
	Code    int           `json:"code"`
	Message string        `json:"message"`
	Results *CallRespData `json:"results"`
	RowID   int           `json:"rowid"`
}

type CallRespData struct {
	Total int         `json:"total"`
	Data  interface{} `json:"data"`
}

// 4.0 路由对象专用结构 (兼容 IPv4/IPv6 分组)
// Specific structures for 4.0 route_object (compatible with IPv4/IPv6 groups)
type routeObject4 struct {
	ID         int    `json:"id"`
	GroupName  string `json:"group_name"`
	Type       int    `json:"type"` // 0: IPv4, 1: IPv6
	GroupValue []map[string]string `json:"group_value"` // 包含 "ip" 或 "ipv6" 键
	Comment    string `json:"comment"`
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

// Login 爱快 4.0 登录
// iKuai 4.0 login
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
	if resp.Code != 0 {
		return errors.New(resp.Message)
	}
	return nil
}
