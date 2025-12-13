# 短信监听
dbus-monitor --system "type='signal',interface='org.ofono.MessageManager',member='IncomingMessage'"
# 如果无法监听短信，请查询是CNMI的第二个参数是否为2
# AT+CNMI: 3,2,0,1,
AT+CNMI=3,2,0,1,0
dbus-send --system --print-reply --dest=org.ofono /ril_0 org.ofono.Modem.SendAtcmd string:"AT+CNMI?"
dbus-send --system --print-reply --dest=org.ofono /ril_0 org.ofono.Modem.SendAtcmd string:"AT+CNMI=3,2,0,1,0"

# 发送短信
dbus-send --system --dest=org.ofono --print-reply /ril_0 org.ofono.MessageManager.SendMessage string:"phone_num" string:"sms_content"
# 发送AT指令   #单独写个c程序，打印原始内容
dbus-send --system --print-reply --dest=org.ofono /ril_0 org.ofono.Modem.SendAtcmd string:"at"
# 获取当前基站信息
dbus-send --system --dest=org.ofono --print-reply /ril_0 org.ofono.NetworkMonitor.GetServingCellInformation

dbus-send --system --dest=org.ofono --print-reply /ril_0 org.ofono.NetworkMonitor.GetCellsInformation

# nr当前基站
# AT+SPENGMD=0,14,1
# nr邻区基站
# AT+SPENGMD=0,14,2

# lte当前基站
# AT+SPENGMD=0,6,0
# lte邻区基站
# AT+SPENGMD=0,6,6

# 限速查询 两个都可以
# AT+CGEQOSRDP=1
# AT+C5QOSRDP


# 获取网络模式
    # {0,  "WCDMA preferred"}, {1,  "GSM only"}, {2,  "WCDMA only"},
    # {3,  "GSM/WCDMA auto"}, {4,  "CDMA/EvDo auto"}, {5,  "CDMA only"},
    # {6,  "EvDo only"}, {7,  "GSM/WCDMA/CDMA/EvDo auto"}, {8,  "LTE/CDMA/EvDo auto"},
    # {9,  "LTE/GSM/WCDMA auto"}, {10, "LTE/CDMA/EvDo/GSM/WCDMA auto"}, {11, "LTE only"},
    # {12, "LTE/WCDMA auto"}, {13, "NR 5G/LTE/GSM/WCDMA auto"}, {14, "NR 5G only"},
    # {15, "NR 5G/LTE auto"}
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.RadioSettings.GetProperties
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.RadioSettings.SetProperty string:"TechnologyPreference" variant:string:"NR 5G/LTE auto"
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.RadioSettings.SetProperty string:"TechnologyPreference" variant:string:"NR 5G only"
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.RadioSettings.SetProperty string:"TechnologyPreference" variant:string:"LTE only"
#切换模式如果没网络就Active
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0/context1 org.ofono.ConnectionContext.SetProperty string:"Active" variant:boolean:true
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0/context1 org.ofono.ConnectionContext.SetProperty string:"Active" variant:boolean:false

# 获取激活状态
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0/context1 org.ofono.ConnectionContext.GetProperties
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.ConnectionManager.GetContexts
    #   dict entry(
    #      string "Active"
    #      variant             boolean true
    #   )

# 获取object path
# dbus-send --system --dest=org.ofono --type=method_call --print-reply / org.ofono.Manager.GetModems
# dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.Modem.GetProperties


#获取短信中心号码
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.MessageManager.GetProperties
#    array [
#       dict entry(
#          string "ServiceCenterAddress"
#          variant             string "+8613800200569"
#       )
#       dict entry(
#          string "UseDeliveryReports"
#          variant             boolean false
#       )
#       dict entry(
#          string "Bearer"
#          variant             string "cs-preferred"
#       )
#       dict entry(
#          string "Alphabet"
#          variant             string "default"
#       )
#    ]
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.MessageManager.SetProperty string:"ServiceCenterAddress" variant:string:"+8613800200569"


# 运营商name
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.NetworkRegistration.GetProperties
    #   dict entry(
    #      string "Name"
    #      variant             string "CMCC"
    #   )

 

# 1 发送短信
# 2 接收短信
# 3 查询短信，删除短信
# 4 发送at
# 5 获取网络模式
# 6 设置网络模式
# 7 获取短信中心号码
# 8 设置短信中心号码
# 9 锁频使用官方
# 10 at当前基站信息解析
# 11 邻区信息解析
# 12 GetServingCellInformation 当前基站信息解析
# 13 查询锁小区
# 14 锁小区
# 15 ConnectionContext激活查询
# 16 ConnectionContext激活
# 17 获取运营商名称


# 查询imei AT+GSN - imsi AT+CIMI - iccid AT+CCID ，签约

# 一些系统信息查询

# cpu占用
# 温度
# 内存
# uptime

# sim信息
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.SimManager.GetProperties
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.SimManager.GetProperties

dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.Modem.SetProperty string:"Online" variant:boolean:true
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.Modem.SetProperty string:"Online" variant:boolean:false
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.Modem.GetProperties 

dbus-send --system --print-reply --dest=org.ofono /ril_0 org.ofono.Modem.SendAtcmd string:"at+spcapability=51,1,132"
dbus-send --system --print-reply --dest=org.ofono /ril_0 org.ofono.Modem.SendAtcmd string:"at+spcapability=51,0"




AT+CGEQOSRDP=1;+SPCAPABILITY=51,0;+SPTESTMODE?;+CNMI?;+CGCONTRDP=1;+CCID;+SPIMEI?


# Status registered
dbus-send --system --print-reply --dest=org.ofono /ril_0 org.ofono.NetworkRegistration.GetProperties
# get Online true or false
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.Modem.GetProperties
    # 如果为false
    dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0/context1 org.ofono.ConnectionContext.SetProperty string:"Active" variant:boolean:true
# set Online true
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.Modem.SetProperty string:"Online" variant:boolean:true
# set Online false
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.Modem.SetProperty string:"Online" variant:boolean:false




dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.ConnectionManager.SetInitAPN string:"3gnet,,,2,2"
dbus-send --system --dest=org.ofono --type=method_call --print-reply /ril_0 org.ofono.ConnectionManager.SetInitAPN string:"3gnet,,,2,0"