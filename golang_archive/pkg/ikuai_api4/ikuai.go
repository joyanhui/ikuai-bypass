package ikuai_api4

import (
	"crypto/md5"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"fmt"
	"ikuai-bypass/pkg/ikuai_common"
	"ikuai-bypass/pkg/logger"
	"net/http"
	"net/http/cookiejar"
	"time"
)

const FUNC_NAME_ROUTE_OBJECT = "route_object"

type IKuai struct {
	baseurl string
	client  *http.Client
	L       *logger.Logger
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
	ID         int                 `json:"id"`
	GroupName  string              `json:"group_name"`
	Type       int                 `json:"type"`        // 0: IPv4, 1: IPv6
	GroupValue []map[string]string `json:"group_value"` // 包含 "ip" 或 "ipv6" 键
	Comment    string              `json:"comment"`
}

type ipGroupObject4 struct {
	Type   int    `json:"type"`
	Gid    string `json:"gid"`
	GpName string `json:"gp_name"`
}

func md5String(v string) string {
	d := []byte(v)
	m := md5.New()
	m.Write(d)
	return hex.EncodeToString(m.Sum(nil))
}

func NewIKuai(baseurl string) *IKuai {
	cookieJar, _ := cookiejar.New(nil)
	return &IKuai{
		baseurl: baseurl,
		client:  &http.Client{Jar: cookieJar, Timeout: time.Second * 10},
		L:       logger.NewLogger("API:iKuaiAPI"),
	}
}

// resolveIpGroupObjects 根据名称列表解析为 iKuai 4.0 的 IP 分组对象
// resolveIpGroupObjects resolves a list of names to iKuai 4.0 IP group objects
func (i *IKuai) resolveIpGroupObjects(names []string) []ipGroupObject4 {
	if len(names) == 0 {
		return []ipGroupObject4{}
	}

	// 获取所有 IP 分组以获取其 ID
	groups, err := i.ShowIpGroupByTagName("")
	if err != nil {
		i.L.Error("API:查询失败", "Failed to query IP groups for resolution: %v", err)
		return []ipGroupObject4{}
	}

	nameMap := make(map[string]ikuai_common.IpGroupData)
	for _, g := range groups {
		nameMap[g.GroupName] = g
	}

	var result []ipGroupObject4
	for _, name := range names {
		if g, ok := nameMap[name]; ok {
			result = append(result, ipGroupObject4{
				Type:   0, // IPv4
				Gid:    fmt.Sprintf("IPGP%d", g.ID),
				GpName: g.GroupName,
			})
		}
	}
	return result
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
