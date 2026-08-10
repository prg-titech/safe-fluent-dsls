package prg.titech.sql;

@SuppressWarnings("unused") public class QueryBuilder {
    private final String[] selections;
    private String source;
    private WhereBuilder whereClause;

    public QueryBuilder(String[] selections) {
        this.selections = selections;
    }

    public QueryBuilder from(String source) {
        this.source = source;
        return this;
    }

    public QueryBuilder where(String where) {
        this.whereClause = new WhereBuilder(where);
        return this;
    }

    public QueryBuilder where(Where where) {
        this.whereClause = new WhereBuilder(where.raw());
        return this;
    }

    public QueryBuilder where() {
        this.whereClause = new WhereBuilder();
        return this;
    }

    public QueryBuilder eq() {
        whereClause.eq();
        return this;
    }

    public QueryBuilder ne() {
        whereClause.ne();
        return this;
    }

    public QueryBuilder lt() {
        whereClause.lt();
        return this;
    }

    public QueryBuilder le() {
        whereClause.le();
        return this;
    }

    public QueryBuilder gt() {
        whereClause.gt();
        return this;
    }

    public QueryBuilder ge() {
        whereClause.ge();
        return this;
    }

    public QueryBuilder columnId(String id) {
        whereClause.columnId(id);
        return this;
    }

    public QueryBuilder value(@Quote("\"") String v) {
        whereClause.value(v);
        return this;
    }

    public QueryBuilder and() {
        whereClause.and();
        return this;
    }

    public QueryBuilder and(Where where) {
        whereClause.and(where);
        return this;
    }

    public QueryBuilder or() {
        whereClause.or();
        return this;
    }

    public QueryBuilder or(Where where) {
        whereClause.or(where);
        return this;
    }

    public Query build() {
        if (this.whereClause == null) {
            return new Query(this.selections, this.source);
        } else {
            return new Query(this.selections, this.source, this.whereClause.build());
        }
    }

}
