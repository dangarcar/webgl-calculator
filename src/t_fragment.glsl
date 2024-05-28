#version 300 es

#define SHADER_TEST

#define PUSH(x) { stack[stackTop] = x; stackTop++; }
#define POP(out) { out = stack[stackTop-1]; stackTop--; }
#define BINARY_OP(op) { float a,b; POP(b); POP(a); PUSH(a op b); }
#define UNARY_OP(op) { float a; POP(a); PUSH( (op(a)) ); }

//MEMORY OPERATORS
#define OP_ST_EXPR 0
#define OP_PUSH 1
#define OP_PUSH_X 2
#define OP_PUSH_Y 3
#define OP_CPY 4
#define OP_POP 5
#define OP_STORE 6
#define OP_RET 7

//BINARY OPERATORS
#define OP_ADD (32 | 0)
#define OP_MUL (32 | 1)
#define OP_DIV (32 | 2)
#define OP_POW (32 | 3)

//UNARY OPERATORS
#define OP_MINUS (64 | 0)
#define OP_SIN   (64 | 1)
#define OP_COS   (64 | 2)
#define OP_FLOOR (64 | 3)
#define OP_ABS   (64 | 4)
#define OP_CEIL  (64 | 5)
#define OP_LOG   (64 | 6)
#define OP_LN    (64 | 7)
#define OP_SQRT  (64 | 8)
#define OP_TAN   (64 | 9)

//Global parameters
#define TEO_WIDTH 1.0
#define AA 2
#define SIDE %side%
#define MAX_EXPR 32
#define MAX_STACK_SIZE 128

precision highp float;
precision highp int;

uniform ivec2 origin;
uniform int squareMant, squareExp, squareSize;
uniform int maxExpr;
uniform vec4 expressions[MAX_EXPR];

const int WIDTH = int(TEO_WIDTH*float(AA)); 

const float PI = 3.141592653589793115997963468544185161590576171875;
const float PI_2 = PI/2.0;
const float TAU = 2.0*PI;
const float E = 2.718281828459045090795598298427648842334747314453125;
const float ONE = 1.0;
const float ZERO = 0.0;
const float LN10 = 2.30258509299;

float fadd(float x, float y);
float fsub(float x, float y);
float fmul(float x, float y);
float fdiv(float x, float y);
float fmod(float x, float y);

float flog(float x);
float fln(float x);

float fminus(float x);
float fexp(float x);
float ffloor(float x);
float fceil(float x);
float fabs(float x);
bool fneg(float x);

float fsin(float x);
float fcos(float x);

struct evalColors {
    bool negs[MAX_EXPR];
    int dens[MAX_EXPR];
};

uniform sampler2D u_texture;
struct Instruction {
    int code;
    float arg;
};
Instruction getInstruction(int i) {
    vec4 p = texelFetch(u_texture, ivec2(0, i), 0);
    return Instruction(int(p.r), p.g);
}

uniform int jumpTable[MAX_EXPR];
uniform int programLength; 
ivec2 eval(ivec2 p, int opt) {
    float pixel = (float(squareMant) * pow(10.0, float(squareExp))) / float(squareSize); 
    float unit = pixel/float(AA);
    float x = float(p.x)*unit, y = float(p.y)*unit;

    float stack[MAX_STACK_SIZE];
    int stackTop = 0;
    int programCounter;
    ivec2 ret;

    for(programCounter=jumpTable[opt]; programCounter<programLength; programCounter++) {
        Instruction ins = getInstruction(programCounter);

        if(ins.code == OP_RET) 
            break;
        
        switch(ins.code) {
        /*case OP_ST_EXPR:
            currentExpr = int(ins.arg) & MAX_STACK_SIZE;
            break;*/
        
        case OP_PUSH:
            PUSH(ins.arg);
            break;
        
        case OP_PUSH_X:
            PUSH(x);
            break;
        
        case OP_PUSH_Y:
            PUSH(y);
            break;
        
        case OP_CPY:
            PUSH(stack[stackTop-1]);
            break;
        
        case OP_POP:
            POP(float _popped);
            break;
        
        case OP_STORE:
            ret.x = int(fneg(stack[stackTop-1]));
            break;

        case OP_ADD:
            BINARY_OP(+);
            break;
        
        case OP_MUL:
            BINARY_OP(*);
            break;
        
        case OP_MINUS:
            UNARY_OP(-);
            break;
        }
    }

    return ret;
}

bool line(ivec2 p, int opt) {
    ivec2 a = eval(p + ivec2(-WIDTH, -WIDTH), opt);
    ivec2 b = eval(p + ivec2(WIDTH+1, WIDTH+1), opt);
    ivec2 c = eval(p + ivec2(-WIDTH, WIDTH+1), opt);
    ivec2 d = eval(p + ivec2(WIDTH+1, -WIDTH), opt);

    int g = a.x + b.x + c.x + d.x;
    bool denominators = a.y == b.y && b.y == c.y && c.y == d.y;

    return 0 < g && g < 4 && denominators;
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

out vec4 fragColor;

void main() {
    ivec2 p = ivec2(
        int(gl_FragCoord.x) - origin.x, 
        int(gl_FragCoord.y) + origin.y - SIDE
    ) * AA;
    vec4 color = vec4(0.0, 0.0, 0.0, 0.0);

    for(int i=0; i<maxExpr; i++) {
        if(expressions[i].a < 0.9) continue;
        
        vec3 rgbColor = expressions[i].rgb;
        color = blend(color, lineColor(p, i, rgbColor));
    }

    fragColor = color;
}

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
    float res = fsub(x, fmul(y, ffloor(fdiv(x, y))));
    if(res < 0.0)
        res = fadd(res, y);
    return res;
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
    if(fneg(x)) 
        return fminus(x);
    return x;
}
bool fneg(float x) { //TODO:
    if(x < 0.0) 
        return true;
    return false;
}

const float[] atanInverse2n = float[32](0.7853981633974483, 0.4636476090008061, 0.24497866312686414, 0.12435499454676144, 0.06241880999595735, 0.031239833430268277, 0.015623728620476831, 0.007812341060101111, 0.0039062301319669718, 0.0019531225164788188, 0.0009765621895593195, 0.0004882812111948983, 0.00024414062014936177, 0.00012207031189367021, 6.103515617420877e-05, 3.0517578115526096e-05, 1.5258789061315762e-05, 7.62939453110197e-06, 3.814697265606496e-06, 1.907348632810187e-06, 9.536743164059608e-07, 4.7683715820308884e-07, 2.3841857910155797e-07, 1.1920928955078068e-07, 5.960464477539055e-08, 2.9802322387695303e-08, 1.4901161193847655e-08, 7.450580596923828e-09, 3.725290298461914e-09, 1.862645149230957e-09, 9.313225746154785e-10, 4.656612873077393e-10);
const float[] inverse2n = float[32](1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125, 0.00390625, 0.001953125, 0.0009765625, 0.00048828125, 0.000244140625, 0.0001220703125, 6.103515625e-05, 3.0517578125e-05, 1.52587890625e-05, 7.62939453125e-06, 3.814697265625e-06, 1.9073486328125e-06, 9.5367431640625e-07, 4.76837158203125e-07, 2.384185791015625e-07, 1.1920928955078125e-07, 5.960464477539063e-08, 2.9802322387695312e-08, 1.4901161193847656e-08, 7.450580596923828e-09, 3.725290298461914e-09, 1.862645149230957e-09, 9.313225746154785e-10, 4.656612873077393e-10);

vec2 cordic(float theta) {
    float angle = theta;
    float x = 0.6072529350088812;
    float y = 0.0;
    float change = 1.0;

    for(int i=0; i<32; ++i) {
        float d = (angle > 0.0)? 1.0 : -1.0;

        float dx = d * y * inverse2n[i];
        float dy = d * x * inverse2n[i];

        x -= dx;
        y += dy;
        angle -= d * atanInverse2n[i];
        change = abs(dx) + abs(dy);
    }

    return vec2(x, y);
}

float fsin(float x) {
    float xmodpi2 = fmod(x, PI_2);
    float xmodpi = fmod(x, PI);
    float xmodtau = fmod(x, TAU);

    if(xmodpi > xmodpi2) {
        x = PI_2 - xmodpi2;
    } else {
        x = xmodpi2;
    }
    
    float result = cordic(x).y;
    
    if(xmodtau > xmodpi) {
        return -result;
    } else {
        return result;
    }
}

float fcos(float x) {
    float xmodpi2 = fmod(x, PI_2);
    float xmodpi = fmod(x, PI);
    float xmodtau = fmod(x, TAU);

    if(xmodpi > xmodpi2) {
        x = PI_2 - xmodpi2;
    } else {
        x = xmodpi2;
    }

    float result = cordic(x).x;

    if(xmodtau > PI_2 && xmodtau < TAU-PI_2)
        return -result;
    else
        return result;
}