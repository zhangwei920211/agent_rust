"""独立核验资产登记表 xlsx 产物结构与内容（只读）。"""
import re
import zipfile

p = "D:/AgentRust/雁能1#_资产登记表.xlsx"
z = zipfile.ZipFile(p)
print("valid xlsx, members:", len(z.namelist()))
print("has sheet1:", "xl/worksheets/sheet1.xml" in z.namelist())
ss = z.read("xl/sharedStrings.xml").decode("utf-8")
strs = re.findall(r"<t[^>]*>(.*?)</t>", ss)
print("sharedStrings count:", len(strs))
print("表头+前几个:", strs[:8])
targets = ("安全型五孔插座", "感烟火灾探测器", "双管格栅灯", "照明配电箱", "数据插座")
print("含设备名样本:", [s for s in strs if s in targets])
sheet = z.read("xl/worksheets/sheet1.xml").decode("utf-8")
m = re.search(r'<dimension ref="([^"]+)"', sheet)
print("dimension:", m.group(1) if m else "n/a")
