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

type StreamIpPortData struct {
	Protocol  string `json:"protocol"`
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
	
	ShowCustomIspByComment() ([]CustomIspData, error)
	AddCustomIsp(name, tag, ipgroup string) error
	DelCustomIsp(id string) error
	GetCustomIspAll(tag string) (string, error)
	DelCustomIspFromPreIds(preIds string) error
	DelCustomIspAll(cleanTag string) error

	ShowStreamDomainByComment(comment string) ([]StreamDomainData, error)
	AddStreamDomain(iface, tag, srcAddr, domains string) error
	DelStreamDomain(id string) error
	GetStreamDomainAll(tag string) (string, error)
	DelStreamDomainFromPreIds(preIds string) error
	DelStreamDomainAll(cleanTag string) error

	ShowIpGroupByComment(comment string) ([]IpGroupData, error)
	ShowIpGroupByName(name string) ([]IpGroupData, error)
	AddIpGroup(groupName, addrPool string) error
	DelIpGroup(id string) error
	GetIpGroup(tag string) (string, error)
	DelIKuaiBypassIpGroup(cleanTag string) error
	GetAllIKuaiBypassIpGroupNamesByName(name string) ([]string, error)

	ShowIpv6GroupByComment(comment string) ([]Ipv6GroupData, error)
	ShowIpv6GroupByName(name string) ([]Ipv6GroupData, error)
	AddIpv6Group(groupName, addrPool string) error
	DelIpv6Group(id string) error
	GetIpv6Group(tag string) (string, error)
	DelIKuaiBypassIpv6Group(cleanTag string) error
	GetAllIKuaiBypassIpv6GroupNamesByName(name string) ([]string, error)

	AddStreamIpPort(forwardType string, iface string, dstAddr string, srcAddr string, nexthop string, tag string, mode int, ifaceband int) error
	ShowStreamIpPortByComment(comment string) ([]StreamIpPortData, error)
	DelStreamIpPort(id string) error
	DelIKuaiBypassStreamIpPort(cleanTag string) error
	GetStreamIpPortIdsByTag(tag string) (string, error)
}
