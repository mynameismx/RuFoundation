# Generated by Django 4.1.2 on 2022-12-01 10:48

from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    dependencies = [
        ('web', '0026_alter_article_updated_at_alter_articlelogentry_type_and_more'),
    ]

    operations = [
        migrations.CreateModel(
            name='TagsCategory',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('name', models.TextField(verbose_name='Полное название')),
                ('description', models.TextField(blank=True, verbose_name='Описание')),
                ('priority', models.PositiveIntegerField(null=True, blank=True, verbose_name='Порядковый номер')),
                ('slug', models.TextField(unique=True, verbose_name='Наименование')),
            ],
            options={
                'verbose_name': 'Категория тегов',
                'verbose_name_plural': 'Категории тегов',
            },
        ),
        migrations.RemoveConstraint(
            model_name='tag',
            name='web_tag_unique',
        ),
        migrations.RemoveIndex(
            model_name='tag',
            name='web_tag_name_41cd6e_idx',
        ),
        migrations.AlterField(
            model_name='tag',
            name='name',
            field=models.TextField(verbose_name='Название'),
        ),
        migrations.AddField(
            model_name='tagscategory',
            name='site',
            field=models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, to='web.site', verbose_name='Сайт'),
        ),
        migrations.AddField(
            model_name='tag',
            name='category',
            field=models.ForeignKey(blank=True, null=True, on_delete=django.db.models.deletion.SET_NULL, to='web.tagscategory', verbose_name='Категория'),
        ),
        migrations.AddIndex(
            model_name='tag',
            index=models.Index(fields=['category', 'name'], name='web_tag_categor_b8c5d5_idx'),
        ),
        migrations.AddConstraint(
            model_name='tag',
            constraint=models.UniqueConstraint(fields=('site', 'category', 'name'), name='web_tag_unique'),
        ),
        migrations.AddIndex(
            model_name='tagscategory',
            index=models.Index(fields=['name'], name='web_tagscat_name_19e56e_idx'),
        ),
        migrations.AddConstraint(
            model_name='tagscategory',
            constraint=models.UniqueConstraint(fields=('site', 'slug'), name='web_tagscategory_unique'),
        ),
    ]
