#include <metal_stdlib>
using namespace metal;
struct Vertex { float4 position; float4 normal; float4 color; float4 material; };
struct Camera { float4 eye; float4 right; float4 up; float4 forward; float4 projection; };
struct Raster { float4 position [[position]]; float3 normal; float4 color; float glow; float3 view; };
vertex Raster solid_vertex(uint id [[vertex_id]], const device Vertex *v [[buffer(0)]], constant Camera &c [[buffer(1)]]) {
    Raster o;
    float3 delta=v[id].position.xyz-c.eye.xyz;
    float depth=dot(delta,c.forward.xyz);
    float f=c.projection.x, aspect=c.projection.y, near=c.projection.z, far=c.projection.w;
    o.position=float4(dot(delta,c.right.xyz)*f/aspect,dot(delta,c.up.xyz)*f,far/(far-near)*depth-far*near/(far-near),depth);
    o.normal=v[id].normal.xyz; o.color=v[id].color; o.glow=v[id].material.x;
    o.view=-delta;
    return o;
}
fragment float4 solid_fragment(Raster in [[stage_in]]) {
    float3 n=normalize(in.normal), light=normalize(float3(-0.4,0.8,0.6));
    float diffuse=0.28+0.72*max(dot(n,light),0.0);
    float rim=pow(1.0-abs(dot(n,normalize(in.view))),2.0);
    float3 lit=in.color.rgb*diffuse + in.color.rgb*in.glow*(0.55+0.8*rim);
    return float4(lit,in.color.a);
}
