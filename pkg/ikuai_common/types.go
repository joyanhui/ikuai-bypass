package ikuai_common

type CustomIspData struct {
	Ipgroup string `json:"ipgroup"`
	Time    string `json:"time"`
	ID      int    `json:"id"`
	Comment string `json:"comment"`
	Name    string `json:"name"`
}

type StreamDomainData struct {
	Week      string `json:"week"`
	Comment   string `json:"comment"`
	TagName   string `json:"tagname"`
	Domain    string `json:"domain"`
	SrcAddr   string `json:"src_addr"`
	Interface string `json:"interface"`
	Time      string `json:"time"`
	ID        int    `json:"id"`
	Enabled   string `json:"enabled"`
}

type IpGroupData struct {
	AddrPool  string `json:"addr_pool"`
	Comment   string `json:"comment"`
	GroupName string `json:"group_name"`
	ID        int    `json:"id"`
	Type      int    `json:"type"`
}

type Ipv6GroupData struct {
	AddrPool  string `json:"addr_pool"`
	Comment   string `json:"comment"`
	GroupName string `json:"group_name"`
	ID        int    `json:"id"`
	Type      int    `json:"type"`
}

type DomainGroupData struct {
	GroupName string `json:"group_name"`
	GroupID   string `json:"group_id"`
	ID        int    `json:"id"`
	Type      int    `json:"type"`
	Comment   string `json:"comment"`
	Domains   string `json:"domains"`
}

type StreamIpPortData struct {
	Protocol  string `json:"protocol"`
	TagName   string `json:"tagname"`
	SrcPort   string `json:"src_port"`
	ID        int    `json:"id"`
	Enabled   string `json:"enabled"`
	Week      string `json:"week"`
	Comment   string `json:"comment"`
	Time      string `json:"time"`
	Nexthop   string `json:"nexthop"`
	IfaceBand int    `json:"iface_band"`
	Interface string `json:"interface"`
	Mode      int    `json:"mode"`
	SrcAddr   string `json:"src_addr"`
	DstPort   string `json:"dst_port"`
	DstAddr   string `json:"dst_addr"`
	Type      int    `json:"type"`
}

type IKuaiClient interface {
	Login(username, password string) error

	ShowCustomIspByTagName(tagName string) ([]CustomIspData, error)
	AddCustomIsp(tag, ipgroup string, index int) error
	EditCustomIsp(tag, ipgroup string, index int, id int) error
	DelCustomIsp(id string) error
	GetCustomIspAll(tag string) (string, error)
	GetCustomIspMap(tag string) (map[int]int, error)
	DelCustomIspFromPreIds(preIds string) error
	DelCustomIspAll(cleanTag string) error

	ShowStreamDomainByTagName(tagName string) ([]StreamDomainData, error)
	AddStreamDomain(iface, tag, srcAddr, srcAddrOptIpGroup, domains string, index int) error
	EditStreamDomain(iface, tag, srcAddr, srcAddrOptIpGroup, domains string, index int, id int) error
	DelStreamDomain(id string) error
	GetStreamDomainAll(tag string) (string, error)
	GetStreamDomainMap(tag string) (map[int]int, error)
	DelStreamDomainFromPreIds(preIds string) error
	DelStreamDomainAll(cleanTag string) error

	ShowIpGroupByTagName(tagName string) ([]IpGroupData, error)
	ShowIpGroupByName(name string) ([]IpGroupData, error)
	AddIpGroup(tag, addrPool string, index int) error
	EditIpGroup(tag, addrPool string, index int, id int) error
	DelIpGroup(id string) error
	GetIpGroup(tag string) (string, error)
	GetIpGroupMap(tag string) (map[int]int, error)
	DelIKuaiBypassIpGroup(cleanTag string) error
	GetAllIKuaiBypassIpGroupNamesByName(name string) ([]string, error)

	ShowIpv6GroupByTagName(tagName string) ([]Ipv6GroupData, error)
	ShowIpv6GroupByName(name string) ([]Ipv6GroupData, error)
	AddIpv6Group(tag, addrPool string, index int) error
	EditIpv6Group(tag, addrPool string, index int, id int) error
	DelIpv6Group(id string) error
	GetIpv6Group(tag string) (string, error)
	GetIpv6GroupMap(tag string) (map[int]int, error)
	DelIKuaiBypassIpv6Group(cleanTag string) error
	GetAllIKuaiBypassIpv6GroupNamesByName(name string) ([]string, error)

	ShowDomainGroupByTagName(tagName string) ([]DomainGroupData, error)
	AddDomainGroup(tag, domains string, index int) error
	EditDomainGroup(tag, domains string, index int, id int) error
	DelDomainGroup(id string) error
	GetDomainGroup(tag string) (string, error)
	GetDomainGroupMap(tag string) (map[int]int, error)
	DelIKuaiBypassDomainGroup(cleanTag string) error

	AddStreamIpPort(forwardType string, iface string, dstAddr string, srcAddr string, nexthop string, tag string, mode int, ifaceband int) error
	EditStreamIpPort(forwardType string, iface string, dstAddr string, srcAddr string, nexthop string, tag string, mode int, ifaceband int, id int) error
	ShowStreamIpPortByTagName(tagName string) ([]StreamIpPortData, error)
	DelStreamIpPort(id string) error
	DelIKuaiBypassStreamIpPort(cleanTag string) error
	GetStreamIpPortIdsByTag(tag string) (string, error)
	GetStreamIpPortMap(tag string) (map[string]int, error)

	BuildIndexedTagName(tag string, index int) string
}
