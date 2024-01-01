#version 300 es
precision mediump float;
out vec4 fragColor;

uniform vec4 u_Color;
void main()
{
	fragColor = u_Color;
}
