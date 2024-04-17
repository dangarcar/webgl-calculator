#define TEO_WIDTH 1.0
#define AA 4
#define SIDE %side%
#define MAX_EXPR 256

precision highp float;
precision highp int;

uniform ivec2 origin;
uniform int squareMant, squareExp, squareSize;
uniform int maxExpr;
uniform vec4 expressions[MAX_EXPR];

float PIXEL = (float(squareMant) * pow(10.0, float(squareExp))) / float(squareSize); 
float UNIT = PIXEL/float(AA);
const int WIDTH = int(TEO_WIDTH*float(AA)); 

float fadd(float x, float y);
float fsub(float x, float y);
float fmul(float x, float y);
float fdiv(float x, float y);
float fmod(float x, float y);

float fpow(float base, float ex);
float flog(float x, float base);
float flog(float x);

float fminus(float x);
float fexp(float x);
float fln(float x);
float ffloor(float x);
float fceil(float x);
float fabs(float x);
int fneg(float x);

float fsin(float x);
float fcos(float x);

int eval(ivec2 p, int opt);

/*int eval(ivec2 p, int opt) {
    float x = float(p.x)*UNIT, y = float(p.y)*UNIT;

    if(opt == 0)
        return fneg(fsub(x, y));
    else
        return fneg(fsub((x-float(opt))*(x-float(opt)), y));

    return 0;
}*/

//To change dinamically the eval function
%eval%

bool line(ivec2 p, int opt) {
    int g = eval(p + ivec2(-WIDTH, -WIDTH), opt)
          + eval(p + ivec2(WIDTH+1, WIDTH+1), opt)
          + eval(p + ivec2(-WIDTH, WIDTH+1), opt)
          + eval(p + ivec2(WIDTH+1, -WIDTH), opt);

    return 0 < g && g < 4;
}

vec4 lineColor(ivec2 p, int opt, vec3 rgb) {
    int count = 0;
    for(int i=0; i<AA; ++i) {
        for(int j=0; j<AA; ++j) {
            ivec2 np = p + ivec2(i, j);
            count += int(line(np, opt));
        }
    }

    float alpha = float(count)/float(AA*AA);
    return vec4(rgb*alpha, alpha);
}


vec4 blend(vec4 a, vec4 b) {
    float p = a.a, q = 1.0-p;
    return vec4(
        a.r*p + b.r*q,
        a.g*p + b.g*q,
        a.b*p + b.b*q,
        a.a*p + b.a*q
    );
}

void main() {
    ivec2 p = ivec2(
        int(gl_FragCoord.x) - origin.x, 
        int(gl_FragCoord.y) + origin.y - SIDE
    ) * AA;
    vec4 color = vec4(0.0, 0.0, 0.0, 0.0);

    for(int i=0; i<MAX_EXPR; i++) {
        if(i >= maxExpr) break;
        if(expressions[i].a < 0.9) continue;
        
        vec3 rgbColor = expressions[i].rgb;
        color = blend(color, lineColor(p, i, rgbColor));
    }

    gl_FragColor = color;
}

const float PI = 3.141592653589793115997963468544185161590576171875;
const float TAU = 2.0*PI;
const float E = 2.718281828459045090795598298427648842334747314453125;
const float ONE = 1.0;
const float ZERO = 0.0;
const float LN10 = 2.30258509299;

float fadd(float x, float y) {
    return x + y; //TODO:
}
float fsub(float x, float y) {
    return fadd(x, fminus(y));
}
float fmul(float x, float y) {
    return x * y; //TODO:
}
float fdiv(float x, float y) {
    return x / y; //TODO:
}
float fmod(float x, float y) {
    return fsub(x, fmul(y, ffloor(fdiv(x, y))));
}

float fpow(float base, float ex) {
    return fexp(fmul(ex, fln(base)));
}
float flog(float x) {
    return fdiv(fln(x), LN10);
}
float flog(float x, float base) {
    return fdiv(fln(x), fln(base));
}

float fminus(float x) {
    return -x; //TODO:
}
float fexp(float x) {
    return exp(x); //TODO:
}
float fln(float x) {
    return log(x); //TODO:
}
float ffloor(float x) {
    int n = int(x);
    return float(n);
}
float fceil(float x) {
    float frac = fract(x); //TODO:
    int n = int(x);
    if(frac > 1e-10) n++;
    return float(n);
}
float fabs(float x) {
    if(fneg(x) == 1) 
        return fminus(x);
    return x;
}
int fneg(float x) { //TODO:
    if(x < 0.0) return 1;
    return 0;
}

float fsin(float x) {
    x = x - TAU*floor(x/TAU);
    float value = x;
    float term = x;
    
    for(int i=0; i<12; ++i) {
        term = -term*x*x/float(2*i+2)/float(2*i+3);
        value += term;
    }

    return value;
}

float fcos(float x) {
    return cos(x); //TODO:
}