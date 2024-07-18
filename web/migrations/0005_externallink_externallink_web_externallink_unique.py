# Generated by Django 4.0.6 on 2022-07-25 18:51

from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    dependencies = [
        ('web', '0004_alter_settings_rating_mode'),
    ]

    operations = [
        migrations.CreateModel(
            name='ExternalLink',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('link_from', models.TextField(db_collation = "case_insensitive", verbose_name='Ссылающаяся статья')),
                ('link_to', models.TextField(db_collation = "case_insensitive", verbose_name='Целевая статья')),
                ('link_type', models.TextField(choices=[('include', 'Include'), ('link', 'Link')], verbose_name='Тип ссылки')),
                ('site', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, to='web.site', verbose_name='Сайт')),
            ],
            options={
                'verbose_name': 'Связь',
                'verbose_name_plural': 'Связи',
            },
        ),
        migrations.AddConstraint(
            model_name='externallink',
            constraint=models.UniqueConstraint(fields=('site', 'link_from', 'link_to', 'link_type'), name='web_externallink_unique'),
        ),
    ]
