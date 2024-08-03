#include "../target/include/bindings.h"
#include <assert.h>
#include <stdio.h>

void json_print(Json json);

void json_print(Json json) {
        switch (json.tag) {
        case Array:
                printf("[");
                for (unsigned long i = 0; i < json.array.len; i++) {
                        Json e = json.array.elems[i];
                        json_print(e);
                        if (i < json.array.len - 1)
                                printf(", ");
                }
                printf("]");
                break;
        case Number:
                printf("%f", json.number);
                break;
        case Object:
                printf("{");
                for (unsigned long i = 0; i < json.object.len; i++) {
                        struct JsonString s = json.object.elems[i].key;
                        printf("%s : ", s.buf);
                        json_print(*json.object.elems[i].val);
                        if (i < json.object.len - 1)
                                printf(", ");
                }
                printf("}");
                break;
        case String:
                printf("\"%s\"", json.string.buf);
                break;
        case True:
                printf("true");
                break;
        case False:
                printf("false");
                break;
        case Null:
                printf("null");
                break;
        default:
          break;
        }
}

int main() {
        const char *text =
        "{"
        "    \"true\" : true,"
        "    \"false\" : false,"
        "    \"null\" : null,"
        "    \"array\" : [ 12, 12, {"
        "        \"inner\" : 12"
        "    } ],"
        "    \"object\" : { \"1\" : 1 }"
        "}";

        Json json = json_deserialize(text);

        json_print(json);

        json_free(json);
}
