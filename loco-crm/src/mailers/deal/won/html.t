{% extends "base.t" %}
{% block title %}Deal won{% endblock %}
{% block body %}
<h1>🎉 {{ title }} is won</h1>
<p><strong>{{ amount }}</strong>{% if contact %} with {{ contact }}{% endif %}.</p>
<p><a href="{{ domain }}/deals">Open the pipeline</a></p>
{% endblock %}
