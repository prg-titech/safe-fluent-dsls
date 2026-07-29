package prg.titech.sql;

@SuppressWarnings("use") public class WhereBuilder {
    private final StringBuilder clauseBuilder = new StringBuilder();

    public WhereBuilder() {}

    public WhereBuilder(String clauseBeginning) {
        append(clauseBeginning);
    }

    public WhereBuilder eq() {
        append("=");
        return this;
    }

    public WhereBuilder ne() {
        append("!=");
        return this;
    }

    public WhereBuilder lt() {
        append("<");
        return this;
    }

    public WhereBuilder le() {
        append("<=");
        return this;
    }

    public WhereBuilder gt() {
        append(">");
        return this;
    }

    public WhereBuilder ge() {
        append(">=");
        return this;
    }

    public WhereBuilder columnId(String id) {
        append(id);
        return this;
    }

    public WhereBuilder value(String v) {
        if (v.contains(" ")) {
            append('"' + v + '"');
        } else {
            append(v);
        }
        return this;
    }

    public WhereBuilder and() {
        append("AND");
        return this;
    }

    public WhereBuilder and(Where where) {
        append("AND");
        clauseBuilder.append("(");
        clauseBuilder.append(where.raw());
        removeTrailingSpace();
        clauseBuilder.append(")");
        return this;
    }

    public WhereBuilder or() {
        append("OR");
        return this;
    }

    public WhereBuilder or(Where where) {
        append("OR");
        clauseBuilder.append("(");
        clauseBuilder.append(where.raw());
        removeTrailingSpace();
        clauseBuilder.append(")");
        return this;
    }

    public Where build() {
        return new Where(clauseBuilder.toString());
    }

    private void append(String s) {
        clauseBuilder.append(s);
        clauseBuilder.append(' ');
    }

    private void removeTrailingSpace() {
        int newLength = clauseBuilder.length();
        for (int i = clauseBuilder.length() - 1; i >= 0; i--) {
            if (!Character.isWhitespace(clauseBuilder.charAt(i))) {
                break;
            }
            newLength = i;
        }
        clauseBuilder.setLength(newLength);
    }
}
