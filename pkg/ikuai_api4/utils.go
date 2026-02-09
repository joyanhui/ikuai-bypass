package ikuai_api4

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"ikuai-bypass/pkg/ikuai_common"
)

func postJson(httpClient *http.Client, url string, body interface{}, result interface{}) (err error) {
	bodyByte, err := json.Marshal(body)
	if err != nil {
		return
	}

	req, err := http.NewRequest("POST", url, bytes.NewReader(bodyByte))
	if err != nil {
		return
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := httpClient.Do(req)
	if err != nil {
		return
	}

	err = json.NewDecoder(resp.Body).Decode(result)
	_ = resp.Body.Close()
	return
}

// CategorizeAddrs 将地址列表分类为 custom (IP/范围/CIDR) 和 object (分组名称)
// CategorizeAddrs classifies the address list into custom (IP/Range/CIDR) and object (Group Name)
func CategorizeAddrs(addrs []string) (custom []string, object []string) {
	custom = []string{}
	object = []string{}
	for _, addr := range addrs {
		addr = strings.TrimSpace(addr)
		if addr == "" {
			continue
		}
		// 如果包含点、冒号或连字符，且不以 IKB 前缀开头，则视为 IP/范围/CIDR
		// If it contains dot, colon or hyphen, and doesn't start with IKB prefix, it's considered IP/Range/CIDR
		if strings.HasPrefix(addr, ikuai_common.NAME_PREFIX_IKB) {
			object = append(object, addr)
		} else if strings.ContainsAny(addr, ".:-") {
			custom = append(custom, addr)
		} else {
			// 既不包含特殊字符又不带 IKB 前缀，通常是用户手动创建的 IP 分组名
			// Neither contains special characters nor has IKB prefix, usually a manually created IP group name
			object = append(object, addr)
		}
	}
	return
}

func toStringList(v interface{}) []string {
	if v == nil {
		return []string{}
	}
	switch val := v.(type) {
	case []interface{}:
		res := make([]string, len(val))
		for i, item := range val {
			res[i] = fmt.Sprint(item)
		}
		return res
	case []string:
		return val
	}
	return []string{}
}
