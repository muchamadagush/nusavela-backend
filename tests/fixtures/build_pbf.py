"""Generate a tiny synthetic OSM PBF; no real place or production data."""
from pathlib import Path
import struct

def varint(n):
    result=bytearray()
    while n>=128:
        result.append((n&127)|128); n>>=7
    result.append(n)
    return bytes(result)
def vi(field,n): return varint(field<<3)+varint(n)
def data(field,s): return varint((field<<3)|2)+varint(len(s))+s
def zig(n): return (n<<1)^(n>>63)
def packed(field,nums): return data(field,b''.join(varint(n) for n in nums))
def block(kind,payload):
    blob=data(1,payload)
    header=data(1,kind.encode())+vi(3,len(blob))
    return struct.pack('>I',len(header))+header+blob
strings=['','name','Synthetic Museum','tourism','museum','amenity','hospital','Synthetic Hospital','opening_hours','24/7','emergency','yes']
def node(id,lat,lng,keys=[],values=[]):
    return vi(1,zig(id))+packed(2,keys)+packed(3,values)+vi(8,zig(round(lat*1e7)))+vi(9,zig(round(lng*1e7)))
group=b''.join(data(1,node(i,lat,lng)) for i,lat,lng in [(1,.001,.001),(2,.002,.001),(3,.002,.002)])
group+=data(1,node(10,.003,.003,[1,3],[2,4]))
way=vi(1,10)+packed(2,[1,5,8,10])+packed(3,[7,6,9,11])+packed(8,[zig(n) for n in [1,1,1,-2]])
group+=data(3,way)
primitive=data(1,b''.join(data(1,s.encode()) for s in strings))+data(2,group)
pbf=block('OSMHeader',data(4,b'OsmSchema-V0.6'))+block('OSMData',primitive)
Path(__file__).with_name('synthetic.osm.pbf').write_bytes(pbf)
