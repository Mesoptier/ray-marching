void cmd_union() {
    float d2 = value_stack_data[--value_stack_size];
    float d1 = value_stack_data[--value_stack_size];
    value_stack_data[value_stack_size++] = min(d1, d2);
}