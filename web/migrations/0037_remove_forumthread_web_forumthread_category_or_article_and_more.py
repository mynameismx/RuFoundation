# Generated by Django 5.0.6 on 2024-07-25 13:28

import django.db.models.lookups
from django.conf import settings
from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('web', '0036_externallink_web_externa_link_fr_92155a_idx_and_more'),
        migrations.swappable_dependency(settings.AUTH_USER_MODEL),
    ]

    operations = [
        migrations.RemoveConstraint(
            model_name='forumthread',
            name='web_forumthread_category_or_article',
        ),
        migrations.AddConstraint(
            model_name='forumthread',
            constraint=models.CheckConstraint(check=django.db.models.lookups.LessThanOrEqual(lhs=models.Func('article_id', 'category_id', function='num_nonnulls', output_field=models.IntegerField()), rhs=models.Value(1)), name='web_forumthread_category_or_article'),
        ),
    ]
