Pipeline digest
{% for s in stages %}{{ s.stage }}: {{ s.count }} deals, {{ s.total }}
{% endfor %}
