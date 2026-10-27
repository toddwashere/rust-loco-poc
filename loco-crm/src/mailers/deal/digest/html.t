{% extends "base.t" %}
{% block title %}Pipeline digest{% endblock %}
{% block body %}
<h1>Pipeline digest</h1>
<table>
{% for s in stages %}<tr><td>{{ s.stage }}</td><td>{{ s.count }}</td><td>{{ s.total }}</td></tr>{% endfor %}
</table>
{% endblock %}
