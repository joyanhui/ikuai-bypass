package ikuaiapi

import (
	"bytes"
	"encoding/json"
	"net/http"
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
