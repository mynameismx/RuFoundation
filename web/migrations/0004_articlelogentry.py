# Generated by Django 4.0.3 on 2022-04-08 23:24

from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    dependencies = [
        ('web', '0003_alter_articleversion_rendered'),
    ]

    operations = [
        migrations.CreateModel(
            name='ArticleLogEntry',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('type', models.TextField(choices=[('source', 'Source'), ('title', 'Title'), ('new', 'New')])),
                ('meta', models.JSONField(default={})),
                ('created_at', models.DateTimeField(auto_now_add=True)),
                ('comment', models.TextField(default='')),
                ('article', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, to='web.article')),
            ],
        ),
    ]
