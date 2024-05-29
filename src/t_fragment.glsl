#version 300 es

precision highp float;
precision highp int;

%INCLUDE_INTERPRETED%

//Global parameters
#define TEO_WIDTH 1.0
#define AA 2
#define SIDE %side%
#define MAX_EXPR 32
#define MAX_STACK_SIZE 128

#define PUSH(x) { stack[stackTop] = x; stackTop++; }
#define POP(out) { out = stack[stackTop-1]; stackTop--; }
#define UNARY_OP(op) { float a; POP(a); PUSH( (op(a)) ); }
#define BINARY_OP(op) { float a,b; POP(b); POP(a); PUSH( op(a,b) ); }
#define DIV(den_out) { float a,b; POP(b); POP(a); PUSH(fdiv(a,b)); den_out <<= 1; den_out |= int(fneg(b)); }

//MEMORY OPERATORS
#define OP_RET 0
#define OP_PUSH 1
#define OP_PUSH_X 2
#define OP_PUSH_Y 3
#define OP_CPY 4
#define OP_POP 5
#define OP_STORE 6

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

uniform ivec2 origin;
uniform int squareMant, squareExp, squareSize;
uniform int maxExpr;
uniform vec4 expressions[MAX_EXPR];

const int WIDTH = int(TEO_WIDTH*float(AA)); 

%INCLUDE_MATH%

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

    ivec2 ret;

#ifdef INTERPRETED
    float stack[MAX_STACK_SIZE];
    int stackTop = 0;
    int programCounter;

    for(programCounter=jumpTable[opt]; programCounter<programLength; programCounter++) {
        Instruction ins = getInstruction(programCounter);

        if(ins.code == OP_RET) 
            break;
        
        switch(ins.code) {        
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
            BINARY_OP(fadd);
            break;
        
        case OP_MUL:
            BINARY_OP(fmul);
            break;
        
        case OP_MINUS:
            UNARY_OP(fminus);
            break;

        case OP_DIV: 
            DIV(ret.y);
            break;
        
        case OP_POW:
            BINARY_OP(fpow);
            break;

        case OP_SIN:
            UNARY_OP(fsin);
            break;
        
        case OP_COS:
            UNARY_OP(fcos);
            break;
        case OP_FLOOR:
            UNARY_OP(ffloor);
            break;
        
        case OP_ABS:
            UNARY_OP(fabs);
            break;
        
        case OP_CEIL:
            UNARY_OP(fceil);
            break;
        
        case OP_LOG:
            UNARY_OP(flog);
            break;
        
        case OP_LN:
            UNARY_OP(fln);
            break;
        
        case OP_SQRT:
            UNARY_OP(sqrt);
            break;

        case OP_TAN:
            float a; 
            POP(a);
            vec2 r = cordic(a);
            PUSH(r.x);
            PUSH(r.y);
            DIV(ret.y);
            break;
        
        default:
            break;
        }
    }
#else
    switch(opt) {
        %REPLACE_CODE%
        
    default:
        break;
    }
#endif

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

